use std::{borrow::Cow, io::Read, path::PathBuf};

use anyhow::Result;
use flume::{bounded, Sender};
use log::error;
use once_cell::sync::Lazy;
use regex::Regex;
use rusqlite::{params, Connection};

use crate::{command_info::CommandInfo, utils::fig_path};

pub async fn spawn_history_task() -> Sender<CommandInfo> {
    let (sender, receiver) = bounded(64);
    tokio::task::spawn(async move {
        let history_join = tokio::task::spawn_blocking(|| History::load());

        match history_join.await {
            Ok(Ok(mut history)) => {
                while let Ok(command) = receiver.recv_async().await {
                    if let Err(e) = history.insert_command_history(&command) {
                        error!("Failed to insert command into history: {}", e);
                    }
                }
            }
            Ok(Err(e)) => {
                error!("Failed to load history: {}", e);
            }
            Err(e) => {
                error!("Failed to join history thread: {}", e);
            }
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

pub struct History {
    connection: Connection,
}

impl History {
    pub fn load() -> Result<History> {
        let old_history_path: PathBuf = [fig_path().unwrap(), "history".into()]
            .into_iter()
            .collect();

        let new_history_path: PathBuf = [fig_path().unwrap(), "fig.history".into()]
            .into_iter()
            .collect();

        let mut old_history = Vec::new();

        if old_history_path.exists() && !new_history_path.exists() {
            let mut file = std::fs::File::open(&old_history_path)?;
            let mut file_string = String::new();
            file.read_to_string(&mut file_string)?;

            let re = Regex::new(r"- command: (.*)\n  exit_code: (.*)\n  shell: (.*)\n  session_id: (.*)\n  cwd: (.*)\n  time: (.*)").unwrap();

            old_history = re
                .captures_iter(&file_string)
                .map(|cap| {
                    let shell = if cap[3].is_empty() {
                        None
                    } else {
                        Some(unescape_string(&cap[3]).to_string())
                    };

                    let session_id = if cap[4].is_empty() {
                        None
                    } else {
                        Some(unescape_string(&cap[4]).to_string())
                    };

                    let cwd = if cap[5].is_empty() {
                        None
                    } else {
                        Some(PathBuf::from(unescape_string(&cap[5]).to_string()))
                    };

                    CommandInfo {
                        command: unescape_string(&cap[1]).into(),
                        shell,
                        pid: None,
                        session_id,
                        cwd,
                        time: cap[6].parse().ok(),
                        hostname: None,
                        in_ssh: false,
                        in_docker: false,
                        exit_code: cap[2].parse().ok(),
                    }
                })
                .collect();
        }

        let connection = Connection::open(new_history_path)?;
        let mut history = History { connection };

        create_migrations_table(&mut history.connection)?;
        migrate_history_db(&mut history.connection)?;

        if old_history.len() > 0 {
            for command in old_history {
                history.insert_command_history(&command).ok();
            }
        }

        Ok(history)
    }

    pub fn insert_command_history(&mut self, command_info: &CommandInfo) -> Result<()> {
        self.connection.execute(
            "INSERT INTO history (\
                        command, \
                        shell, \
                        pid, \
                        session_id, \
                        cwd, \
                        time, \
                        in_ssh, \
                        in_docker, \
                        hostname, \
                        exit_code) \
                    VALUES \
                        (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &command_info.command,
                &command_info.shell,
                &command_info.pid,
                &command_info.session_id,
                &command_info
                    .cwd
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned()),
                &command_info.time,
                &command_info.in_ssh,
                &command_info.in_docker,
                &command_info.hostname,
                &command_info.exit_code,
            ],
        )?;
        Ok(())
    }
}

fn create_migrations_table(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS migrations( \
                id INTEGER PRIMARY KEY, \
                version INTEGER NOT NULL, \
                migration_time INTEGER NOT NULL);",
    )?;

    Ok(())
}

fn migrate_history_db(conn: &mut Connection) -> Result<()> {
    let mut max_migration_version_stmt = conn.prepare("SELECT max(version) from migrations;")?;
    let max_migration_version: i64 = max_migration_version_stmt
        .query_row([], |row| row.get(0))
        .unwrap_or(0);

    if max_migration_version < 1 {
        conn.execute_batch(
            "BEGIN; \
                CREATE TABLE IF NOT EXISTS history( \
                    id INTEGER PRIMARY KEY, \
                    command TEXT, \
                    shell TEXT, \
                    pid INTEGER, \
                    session_id TEXT, \
                    cwd TEXT, \
                    time INTEGER , \
                    in_ssh INTEGER, \
                    in_docker INTEGER, \
                    hostname TEXT, \
                    exit_code INTEGER); \
                INSERT INTO migrations(version, migration_time) VALUES (1, strftime('%s', 'now')); \
                COMMIT;",
        )?;
    }

    Ok(())
}
