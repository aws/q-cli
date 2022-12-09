use std::fs::File;
use std::io::{
    BufWriter,
    Write,
};
use std::path::PathBuf;
use std::time::SystemTime;

use fig_util::directories;
use rusqlite::{
    params,
    Connection,
};
use thiserror::Error;
use tracing::{
    error,
    trace,
};

const ALL_COLUMNS: &str = "id, command, shell, pid, session_id, cwd, start_time, duration, hostname, exit_code";

#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SQL error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Directory error: {0}")]
    Directory(#[from] fig_util::directories::DirectoryError),
}

type Result<T, E = Error> = std::result::Result<T, E>;

fn escape_string(s: impl AsRef<str>) -> String {
    s.as_ref()
        .replace('\\', "\\\\")
        .replace('\"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\r', "\\r")
        .replace('\x08', "\\b")
        .replace('\x0c', "\\f")
}

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub command: Option<String>,
    pub shell: Option<String>,
    pub pid: Option<i32>,
    pub session_id: Option<String>,
    pub cwd: Option<String>,
    pub start_time: Option<SystemTime>,
    pub end_time: Option<SystemTime>,
    pub hostname: Option<String>,
    pub exit_code: Option<i32>,
}

pub struct History {
    connection: Connection,
}

impl History {
    pub fn load() -> Result<History> {
        trace!("Loading history");

        let history_path = directories::fig_dir()?.join("fig.history");

        let connection = Connection::open(&history_path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = history_path.metadata()?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&history_path, perms)?;
        }

        let history = History { connection };

        history.migrate()?;

        Ok(history)
    }

    fn migrate(&self) -> Result<()> {
        trace!("Creating migrations table");
        self.connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS migrations( \
                    id INTEGER PRIMARY KEY, \
                    version INTEGER NOT NULL, \
                    migration_time INTEGER NOT NULL \
                );",
        )?;

        trace!("Migrating history database");

        let max_migration_version: i64 = self
            .connection
            .prepare("SELECT max(version) from migrations;")?
            .query_row([], |row| row.get(0))
            .unwrap_or(0);

        let migrate = |n, s| {
            if max_migration_version < n {
                trace!("Running migration {n}");

                self.connection.execute_batch(&format!(
                    "BEGIN; \
                {s} \
                INSERT INTO migrations (version, migration_time) VALUES ({n}, strftime('%s', 'now')); \
                COMMIT;"
                ))
            } else {
                Ok(())
            }
        };

        // Create the initial history table
        migrate(
            1,
            "CREATE TABLE IF NOT EXISTS history( \
                id INTEGER PRIMARY KEY, \
                command TEXT, \
                shell TEXT, \
                pid INTEGER, \
                session_id TEXT, \
                cwd TEXT, \
                time INTEGER, \
                in_ssh INTEGER, \
                in_docker INTEGER, \
                hostname TEXT, \
                exit_code INTEGER \
            );",
        )?;

        // Drop in_ssh and in_docker columns
        migrate(
            2,
            "ALTER TABLE history DROP COLUMN in_ssh; \
            ALTER TABLE history DROP COLUMN in_docker;",
        )?;

        // Rename time -> start_time, add end_time and duration
        migrate(
            3,
            "ALTER TABLE history RENAME COLUMN time TO start_time; \
            ALTER TABLE history ADD COLUMN end_time INTEGER; \
            ALTER TABLE history ADD COLUMN duration INTEGER;",
        )?;

        //

        Ok(())
    }

    pub fn insert_command_history(&self, command_info: &CommandInfo, legacy: bool) -> Result<()> {
        trace!("Inserting command into history: {:?}", command_info);
        // Insert the command into the history table
        // Ensure that the command is not empty
        if let Some(command) = &command_info.command {
            if !command.is_empty() {
                self.connection.execute(
                    "INSERT INTO history 
                        (command, shell, pid, session_id, cwd, start_time, end_time, duration, hostname, exit_code)
                        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                    params![
                        &command_info.command,
                        &command_info.shell,
                        &command_info.pid,
                        &command_info.session_id,
                        &command_info.cwd,
                        &command_info
                            .start_time
                            .as_ref()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs()),
                        &command_info
                            .end_time
                            .as_ref()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|t| t.as_secs()),
                        &command_info
                            .start_time
                            .as_ref()
                            .and_then(|start_time| {
                                command_info
                                    .end_time
                                    .as_ref()
                                    .and_then(|end_time| end_time.duration_since(*start_time).ok())
                            })
                            .map(|duration| duration.as_millis())
                            .and_then(|duration| i64::try_from(duration).ok()),
                        &command_info.hostname,
                        &command_info.exit_code,
                    ],
                )?;
            }
        }

        // Legacy insert into old history file
        if legacy {
            let mut legacy_history_file_opts = File::options();
            legacy_history_file_opts.create(true).append(true);

            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                legacy_history_file_opts.mode(0o600);
            }

            let legacy_history_file = legacy_history_file_opts.open(
                [directories::fig_dir().unwrap(), "history".into()]
                    .into_iter()
                    .collect::<PathBuf>(),
            )?;

            let mut legacy_history_buff = BufWriter::new(legacy_history_file);

            match command_info.command.as_deref() {
                Some(command) if !command.is_empty() => {
                    let exit_code = command_info.exit_code.unwrap_or(0);
                    let shell = command_info.shell.as_deref().unwrap_or("");
                    let session_id = command_info.session_id.as_deref().unwrap_or("");
                    let cwd = command_info.cwd.as_deref().unwrap_or("");
                    let time = command_info
                        .start_time
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs()))
                        .unwrap_or(0);
                    let entry = format!(
                        "\n- command: {}\n  exit_code: {}\n  shell: {}\n  session_id: {}\n  cwd: {}\n  time: {}",
                        escape_string(command),
                        exit_code,
                        escape_string(shell),
                        escape_string(session_id),
                        escape_string(cwd),
                        time
                    );

                    legacy_history_buff.write_all(entry.as_bytes())?;
                    legacy_history_buff.flush()?;
                },
                _ => {},
            }
        }

        Ok(())
    }

    pub fn all_rows(&self) -> Result<Vec<CommandInfo>> {
        let mut stmt = self
            .connection
            .prepare(&format!("SELECT {ALL_COLUMNS} FROM history ORDER BY start_time ASC"))?;

        let rows = stmt.query([])?;

        let rows_mapped = rows.mapped(map_row).collect::<rusqlite::Result<Vec<CommandInfo>>>()?;

        Ok(rows_mapped)
    }

    pub fn rows(&self, limit: usize, offset: usize, order_by: HistoryColumn, order: Order) -> Result<Vec<CommandInfo>> {
        let mut stmt = self.connection.prepare(&format!(
            "SELECT {ALL_COLUMNS} FROM history ORDER BY {order_by} {order} LIMIT ? OFFSET ?",
        ))?;

        let rows = stmt.query(params![limit, offset])?;

        let rows_mapped = rows.mapped(map_row).collect::<rusqlite::Result<Vec<CommandInfo>>>()?;

        Ok(rows_mapped)
    }
}

fn map_row(row: &rusqlite::Row) -> rusqlite::Result<CommandInfo> {
    let start_time = row
        .get::<_, Option<i64>>(6)?
        .and_then(|t| std::time::UNIX_EPOCH.checked_add(std::time::Duration::from_secs(u64::try_from(t).ok()?)));

    let duration = row
        .get::<_, Option<i64>>(7)?
        .and_then(|d| Some(std::time::Duration::from_millis(u64::try_from(d).ok()?)));

    let end_time = start_time
        .as_ref()
        .and_then(|start_time| duration.and_then(|duration| start_time.checked_add(duration)));

    Ok(CommandInfo {
        command: row.get(1)?,
        shell: row.get(2)?,
        pid: row.get(3)?,
        session_id: row.get(4)?,
        cwd: row.get(5)?,
        start_time,
        end_time,
        hostname: row.get(8)?,
        exit_code: row.get(9)?,
    })
}

pub enum HistoryColumn {
    Id,
    Command,
    Shell,
    Pid,
    SessionId,
    Cwd,
    StartTime,
    Duration,
    Hostname,
    ExitCode,
}

impl std::fmt::Display for HistoryColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryColumn::Id => f.write_str("id"),
            HistoryColumn::Command => f.write_str("command"),
            HistoryColumn::Shell => f.write_str("shell"),
            HistoryColumn::Pid => f.write_str("pid"),
            HistoryColumn::SessionId => f.write_str("session_id"),
            HistoryColumn::Cwd => f.write_str("cwd"),
            HistoryColumn::StartTime => f.write_str("start_time"),
            HistoryColumn::Duration => f.write_str("duration"),
            HistoryColumn::Hostname => f.write_str("hostname"),
            HistoryColumn::ExitCode => f.write_str("exit_code"),
        }
    }
}

pub enum Order {
    Asc,
    Desc,
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Order::Asc => f.write_str("ASC"),
            Order::Desc => f.write_str("DESC"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_insert_query() {
        let connection = Connection::open_in_memory().unwrap();
        let history = History { connection };
        history.migrate().unwrap();

        history
            .insert_command_history(
                &CommandInfo {
                    command: Some("fig".into()),
                    shell: Some("bash".into()),
                    pid: Some(123),
                    session_id: Some("session-id".into()),
                    cwd: Some("/home/grant/".into()),
                    start_time: Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(123)),
                    end_time: Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(124)),
                    hostname: Some("laptop".into()),
                    exit_code: Some(0),
                },
                false,
            )
            .unwrap();

        history
            .insert_command_history(
                &CommandInfo {
                    command: Some("cargo test".into()),
                    shell: Some("zsh".into()),
                    pid: Some(124),
                    session_id: Some("session-id".into()),
                    cwd: Some("/home/grant/".into()),
                    start_time: Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(124)),
                    end_time: Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(125)),
                    hostname: Some("laptop".into()),
                    exit_code: Some(0),
                },
                false,
            )
            .unwrap();

        let rows = history.all_rows().unwrap();
        assert_eq!(rows.len(), 2);

        assert_eq!(rows[0].command, Some("fig".into()));
        assert_eq!(rows[0].shell, Some("bash".into()));
        assert_eq!(rows[0].pid, Some(123));
        assert_eq!(rows[0].session_id, Some("session-id".into()));
        assert_eq!(rows[0].cwd, Some("/home/grant/".into()));
        assert_eq!(
            rows[0].start_time,
            Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(123))
        );
        assert_eq!(
            rows[0].end_time,
            Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(124))
        );
        assert_eq!(rows[0].hostname, Some("laptop".into()));
        assert_eq!(rows[0].exit_code, Some(0));

        assert_eq!(rows[1].command, Some("cargo test".into()));
        assert_eq!(rows[1].shell, Some("zsh".into()));
        assert_eq!(rows[1].pid, Some(124));
        assert_eq!(rows[1].session_id, Some("session-id".into()));
        assert_eq!(rows[1].cwd, Some("/home/grant/".into()));
        assert_eq!(
            rows[1].start_time,
            Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(124))
        );
        assert_eq!(
            rows[1].end_time,
            Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(125))
        );
        assert_eq!(rows[1].hostname, Some("laptop".into()));
        assert_eq!(rows[1].exit_code, Some(0));

        let row = history.rows(1, 0, HistoryColumn::Id, Order::Desc).unwrap();
        assert_eq!(row.len(), 1);
        assert_eq!(row[0].command, Some("cargo test".into()));
    }
}
