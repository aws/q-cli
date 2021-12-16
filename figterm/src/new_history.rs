use std::path::PathBuf;

use anyhow::Result;
use rusqlite::{Connection, Statement};

use crate::utils::fig_path;

struct HistoryEntry {
    /// Id of Entry
    pub id: i64,

    /// The raw command run by the user
    pub command: String,
    /// The command with alias resolved
    pub command_unaliased: String,
    /// The top level unaliased command
    pub command_top_level: String,

    /// Shell used for command
    pub shell: String,
    /// Working directory cmd was run in
    pub cwd: String,

    /// Session id
    pub session_id: String,
    /// When the command was run
    pub time_run: u64,
    /// Exit code of the command
    pub exit_code: Option<i32>,
    //
}

pub struct History {
    connection: Connection,
}

impl History {
    pub fn load() -> Result<History> {
        let history_path: PathBuf = [fig_path().unwrap(), "history2".into()]
            .into_iter()
            .collect();

        let mut connection = Connection::open(history_path)?;

        // TODO: MIGRATE HERE
        make_migrations_table(&mut connection)?;
        migrate_history(&mut connection)?;

        Ok(History { connection })
    }
}

fn make_migrations_table(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS migrations( \
                                id INTEGER PRIMARY KEY, \
                                version INTEGER NOT NULL, \
                                migration_time INTEGER NOT NULL);",
    )?;

    Ok(())
}

fn migrate_history(conn: &mut Connection) -> Result<()> {
    conn.query_row("SELECT max(version) from migrations;", [], |row| {
        Ok(())
    });

    Ok(())
}