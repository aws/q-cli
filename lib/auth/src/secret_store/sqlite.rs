#![allow(dead_code)]
use fig_settings::sqlite::{
    database,
    Db,
};

use super::Secret;
use crate::Result;

pub struct SqliteSecretStore {
    db: &'static Db,
}

impl SqliteSecretStore {
    pub async fn new() -> Result<Self> {
        Ok(Self { db: database()? })
    }

    pub async fn set(&self, key: &str, password: &str) -> Result<()> {
        Ok(self.db.set_auth_value(key, password)?)
    }

    pub async fn get(&self, key: &str) -> Result<Option<Secret>> {
        Ok(self.db.get_auth_value(key)?.map(Secret))
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        Ok(self.db.unset_auth_value(key)?)
    }
}
