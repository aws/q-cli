use std::borrow::Cow;
use std::fs::File;
use std::io::{
    BufWriter,
    Read,
    Write,
};
use std::path::PathBuf;

use alacritty_terminal::term::CommandInfo;
use anyhow::Result;
use fig_util::directories;
use flume::{
    bounded,
    Sender,
};
use once_cell::sync::Lazy;
use regex::Regex;
use rusqlite::{
    params,
    Connection,
};
use tracing::{
    error,
    trace,
};

pub async fn spawn_history_task() -> Sender<CommandInfo> {
    trace!("Spawning history task");

    let (sender, receiver) = bounded(64);
    tokio::task::spawn(async move {
        let history_join = tokio::task::spawn_blocking(History::load);

        match history_join.await {
            Ok(Ok(history)) => {
                while let Ok(command) = receiver.recv_async().await {
                    if let Err(e) = history.insert_command_history(&command, true) {
                        error!("Failed to insert command into history: {}", e);
                    }
                }
            },
            Ok(Err(err)) => {
                error!("Failed to load history: {err}");
            },
            Err(err) => {
                error!("Failed to join history thread: {err}");
            },
        }
    });

    sender
}

static UNESCAPE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\\(.)").unwrap());

fn unescape_string(s: &str) -> Cow<str> {
    UNESCAPE_RE.replace_all(s, |caps: &regex::Captures| {
        let c = caps.get(1).unwrap().as_str();
        match c {
            "n" => String::from("\n"),
            "t" => String::from("\t"),
            "\\" => String::from("\\"),
            "\"" => String::from("\""),
            "/" => String::from("/"),
            "b" => String::from("\x08"),
            "r" => String::from("\r"),
            "f" => String::from("\x0c"),
            _ => format!("\\{}", c),
        }
    })
}

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

pub struct History {
    connection: Connection,
}

impl History {
    pub fn load() -> Result<History> {
        trace!("Loading history");

        let old_history_path = directories::fig_dir()?.join("history");

        let history_path: PathBuf = [directories::fig_dir().unwrap(), "fig.history".into()]
            .into_iter()
            .collect();

        let mut old_history = Vec::new();

        let history_exists = history_path.exists();

        let connection = Connection::open(&history_path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = history_path.metadata()?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&history_path, perms)?;
        }

        let history = History { connection };

        if old_history_path.exists() && !history_exists {
            let mut file = File::open(&old_history_path)?;
            let mut file_string = String::new();
            file.read_to_string(&mut file_string)?;

            let re = Regex::new(
                r"- command: (.*)\n  exit_code: (.*)\n  shell: (.*)\n  session_id: (.*)\n  cwd: (.*)\n  time: (.*)",
            )
            .unwrap();

            old_history = re
                .captures_iter(&file_string)
                .map(|cap| {
                    let command = if cap[1].is_empty() {
                        None
                    } else {
                        Some(unescape_string(&cap[1]).trim().to_string())
                    };

                    let shell = if cap[3].is_empty() {
                        None
                    } else {
                        Some(unescape_string(&cap[3]).trim().to_string())
                    };

                    let session_id = if cap[4].is_empty() {
                        None
                    } else {
                        Some(unescape_string(&cap[4]).trim().to_string())
                    };

                    let cwd = if cap[5].is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(unescape_string(&cap[5]).to_string()))
                    };

                    CommandInfo {
                        command,
                        shell,
                        pid: None,
                        session_id,
                        cwd,
                        start_time: cap[6]
                            .parse()
                            .ok()
                            .map(|t: u64| std::time::UNIX_EPOCH + std::time::Duration::from_secs(t)),
                        end_time: None,
                        hostname: None,
                        exit_code: cap[2].parse().ok(),
                    }
                })
                .collect();
        }

        migrate_history_db(&history.connection)?;

        if !old_history.is_empty() {
            for command in old_history {
                history.insert_command_history(&command, false).ok();
            }
        }

        Ok(history)
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
                        &command_info.cwd.as_ref().map(|p| p.to_string_lossy().into_owned()),
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
                &[directories::fig_dir().unwrap(), "history".into()]
                    .into_iter()
                    .collect::<PathBuf>(),
            )?;

            let mut legacy_history_buff = BufWriter::new(legacy_history_file);

            match command_info.command.as_deref() {
                Some(command) if !command.is_empty() => {
                    let exit_code = command_info.exit_code.unwrap_or(0);
                    let shell = command_info.shell.as_deref().unwrap_or("");
                    let session_id = command_info.session_id.as_deref().unwrap_or("");
                    let cwd = command_info
                        .cwd
                        .as_ref()
                        .map(|p| p.to_string_lossy().into_owned())
                        .unwrap_or_else(|| "".to_string());
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
}

fn migrate_history_db(conn: &Connection) -> Result<()> {
    trace!("Creating migrations table");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS migrations( \
                id INTEGER PRIMARY KEY, \
                version INTEGER NOT NULL, \
                migration_time INTEGER NOT NULL \
            );",
    )?;

    trace!("Migrating history database");

    let max_migration_version: i64 = conn
        .prepare("SELECT max(version) from migrations;")?
        .query_row([], |row| row.get(0))
        .unwrap_or(0);

    let migrate = |n, s| {
        if max_migration_version < n {
            trace!("Running migration {n}");

            conn.execute_batch(&format!(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_and_insert() {
        let conn = Connection::open_in_memory().unwrap();
        migrate_history_db(&conn).unwrap();

        let history = History { connection: conn };
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
    }
}
