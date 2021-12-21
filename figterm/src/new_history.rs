use std::path::PathBuf;

use anyhow::Result;
use rusqlite::{Connection, params};

use crate::{command_info::CommandInfo, utils::fig_path};

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
        create_migrations_table(&mut connection)?;
        migrate_history(&mut connection)?;

        Ok(History { connection })
    }

    pub fn insert_command_history(&mut self, command_info: CommandInfo) -> Result<()> {
        self.connection
            .execute(
                "INSERT INTO history (command, shell, pid, session_id, cwd, time, in_ssh, in_docker, hostname, exit_code) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    &command_info.command,
                    &command_info.shell,
                    &command_info.pid,
                    &command_info.session_id,
                    &command_info.cwd.map(|p| p.to_string_lossy().into_owned()),
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

fn migrate_history(conn: &mut Connection) -> Result<()> {
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
