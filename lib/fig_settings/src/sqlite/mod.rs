use std::ops::Deref;
use std::path::{
    Path,
    PathBuf,
};

use fig_util::directories::fig_data_dir;
use once_cell::sync::Lazy;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{
    params,
    Connection,
    Error,
};
use serde_json::Map;
use tracing::{
    debug,
    info,
};

use crate::error::DbOpenError;
use crate::Result;

pub static DATABASE: Lazy<Result<Db, DbOpenError>> = Lazy::new(|| {
    let db = Db::new().map_err(|e| DbOpenError(e.to_string()))?;
    db.migrate().map_err(|e| DbOpenError(e.to_string()))?;
    Ok(db)
});

pub fn database() -> Result<&'static Db, DbOpenError> {
    match DATABASE.as_ref() {
        Ok(db) => Ok(db),
        Err(err) => Err(err.clone()),
    }
}

#[derive(Debug)]
struct Migration {
    name: &'static str,
    sql: &'static str,
}

macro_rules! migrations {
    ($($name:expr),*) => {{
        &[
            $(
                Migration {
                    name: $name,
                    sql: include_str!(concat!("migrations/", $name, ".sql")),
                }
            ),*
        ]
    }};
}

const MIGRATIONS: &[Migration] = migrations![
    "000_migration_table",
    "001_history_table",
    "002_drop_history_in_ssh_docker",
    "003_improved_history_timing",
    "004_state_table"
];

#[derive(Debug)]
pub struct Db {
    pub(crate) pool: Pool<SqliteConnectionManager>,
}

impl Db {
    fn path() -> Result<PathBuf> {
        Ok(fig_data_dir()?.join("data.sqlite3"))
    }

    pub fn new() -> Result<Self> {
        Self::open(&Self::path()?)
    }

    fn open(path: &Path) -> Result<Self> {
        // make the parent dir if it doesnt exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let conn = SqliteConnectionManager::file(path);
        let pool = Pool::builder().build(conn)?;

        // Check the unix permissions of the database file, set them to 0600 if they are not
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(path)?;
            let mut permissions = metadata.permissions();
            if permissions.mode() & 0o777 != 0o600 {
                debug!(?path, "Setting database file permissions to 0600");
                permissions.set_mode(0o600);
                std::fs::set_permissions(path, permissions)?;
            }
        }

        Ok(Self { pool })
    }

    #[cfg(test)]
    pub(crate) fn mock() -> Self {
        let conn = SqliteConnectionManager::memory();
        let pool = Pool::builder().build(conn).unwrap();
        Self { pool }
    }

    pub fn migrate(&self) -> Result<()> {
        let mut conn = self.pool.get()?;
        let transaction = conn.transaction()?;

        // select the max migration id
        let max_id = max_migration(&transaction);

        for (version, migration) in MIGRATIONS.iter().enumerate() {
            // skip migrations that already exist
            match max_id {
                Some(max_id) if max_id >= version as i64 => continue,
                _ => (),
            };

            // execute the migration
            transaction.execute_batch(migration.sql)?;

            info!(%version, name =% migration.name, "Applying migration");

            // insert the migration entry
            transaction.execute(
                "INSERT INTO migrations (version, migration_time) VALUES (?1, strftime('%s', 'now'));",
                params![version],
            )?;
        }

        // commit the transaction
        transaction.commit()?;

        Ok(())
    }

    pub fn get_state_value(&self, key: impl AsRef<str>) -> Result<Option<serde_json::Value>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT value FROM state WHERE key = ?1")?;
        match stmt.query_row([key.as_ref()], |row| row.get(0)) {
            Ok(data) => Ok(Some(data)),
            Err(Error::QueryReturnedNoRows) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub fn set_state_value(&self, key: impl AsRef<str>, value: impl Into<serde_json::Value>) -> Result<()> {
        self.pool
            .get()?
            .execute("INSERT OR REPLACE INTO state (key, value) VALUES (?1, ?2)", params![
                key.as_ref(),
                value.into(),
            ])?;
        Ok(())
    }

    pub fn unset_state_value(&self, key: impl AsRef<str>) -> Result<()> {
        self.pool
            .get()?
            .execute("DELETE FROM state WHERE key = ?1", [key.as_ref()])?;
        Ok(())
    }

    pub fn is_state_value_set(&self, key: impl AsRef<str>) -> Result<bool> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT value FROM state WHERE key = ?1")?;
        match stmt.query_row([key.as_ref()], |_| Ok(())) {
            Ok(()) => Ok(true),
            Err(Error::QueryReturnedNoRows) => Ok(false),
            Err(err) => Err(err.into()),
        }
    }

    pub fn all_state_values(&self) -> Result<Map<String, serde_json::Value>> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare("SELECT key, value FROM state")?;
        let rows = stmt.query_map([], |row| {
            let key = row.get(0)?;
            let value = row.get(1)?;
            Ok((key, value))
        })?;

        let mut map = Map::new();
        for row in rows {
            let (key, value) = row?;
            map.insert(key, value);
        }

        Ok(map)
    }
}

fn max_migration<C: Deref<Target = Connection>>(conn: &C) -> Option<i64> {
    let mut stmt = conn.prepare("SELECT MAX(id) FROM migrations").ok()?;
    stmt.query_row([], |row| row.get(0)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock() -> Db {
        let db = Db::mock();
        db.migrate().unwrap();
        db
    }

    #[test]
    fn test_migrate() {
        let db = mock();

        // assert migration count is correct
        let max_migration = max_migration(&&*db.pool.get().unwrap());
        assert_eq!(max_migration, Some(MIGRATIONS.len() as i64));
    }

    #[test]
    fn list_migrations() {
        // Assert the migrations are in order
        assert!(MIGRATIONS.windows(2).all(|w| w[0].name <= w[1].name));

        // Assert the migrations start with their index
        assert!(
            MIGRATIONS
                .iter()
                .enumerate()
                .all(|(i, m)| m.name.starts_with(&format!("{:03}_", i)))
        );

        // Assert all the files in migrations/ are in the list
        let migration_folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/sqlite/migrations");
        let migration_count = std::fs::read_dir(migration_folder).unwrap().count();
        assert_eq!(MIGRATIONS.len(), migration_count);
    }

    #[test]
    fn json() {
        let db = mock();

        // set
        db.set_state_value("test", "test").unwrap();
        db.set_state_value("int", 1).unwrap();
        db.set_state_value("float", 1.0).unwrap();
        db.set_state_value("bool", true).unwrap();
        db.set_state_value("null", ()).unwrap();
        db.set_state_value("array", vec![1, 2, 3]).unwrap();
        db.set_state_value("object", serde_json::json!({ "test": "test" }))
            .unwrap();
        db.set_state_value("binary", b"test".to_vec()).unwrap();

        // get
        assert_eq!(db.get_state_value("test").unwrap().unwrap(), "test");
        assert_eq!(db.get_state_value("int").unwrap().unwrap(), 1);
        assert_eq!(db.get_state_value("float").unwrap().unwrap(), 1.0);
        assert_eq!(db.get_state_value("bool").unwrap().unwrap(), true);
        assert_eq!(db.get_state_value("null").unwrap().unwrap(), serde_json::Value::Null);
        assert_eq!(
            db.get_state_value("array").unwrap().unwrap(),
            serde_json::json!([1, 2, 3])
        );
        assert_eq!(
            db.get_state_value("object").unwrap().unwrap(),
            serde_json::json!({ "test": "test" })
        );
        assert_eq!(
            db.get_state_value("binary").unwrap().unwrap(),
            serde_json::json!(b"test".to_vec())
        );

        // unset
        db.unset_state_value("test").unwrap();
        db.unset_state_value("int").unwrap();

        // is_set
        assert!(!db.is_state_value_set("test").unwrap());
        assert!(!db.is_state_value_set("int").unwrap());
        assert!(db.is_state_value_set("float").unwrap());
        assert!(db.is_state_value_set("bool").unwrap());
    }

    #[test]
    fn db_open_time() {
        let tempdir = tempfile::tempdir().unwrap();
        let path = tempdir.path().join("data.sqlite3");

        // init the db
        let db = Db::open(&path).unwrap();
        db.migrate().unwrap();
        drop(db);

        let test_count = 100;

        let instant = std::time::Instant::now();
        let db = Db::open(&path).unwrap();
        for _ in 0..test_count {
            db.set_state_value("test", "test").unwrap();
            db.get_state_value("test").unwrap().unwrap();
        }
        let elapsed = instant.elapsed() / test_count;
        println!("time: {:?}", elapsed);
    }
}
