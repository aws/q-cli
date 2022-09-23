use std::fs::File;
use std::future::Future;

use tokio::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::const_new(());

static TEMP_STATE_FILE: Mutex<Option<File>> = Mutex::const_new(None);
static TEMP_SETTINGS_FILE: Mutex<Option<File>> = Mutex::const_new(None);

#[derive(Debug, Default)]
struct DataStore {
    original_env: Option<Vec<(String, String)>>,
    new_env: Option<Vec<(String, String)>>,
}

impl DataStore {
    fn with_env(mut self, env: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>, unset: bool) -> Self {
        self.original_env = Some(std::env::vars().collect());

        if unset {
            for (key, _) in std::env::vars() {
                std::env::remove_var(key);
            }
        }

        let new_env: Vec<_> = env.into_iter().map(|(k, v)| (k.into(), v.into())).collect();
        for (key, value) in &new_env {
            std::env::set_var(key, value);
        }
        self.new_env = Some(new_env);

        self
    }

    fn restore(mut self) {
        if let Some(original_env) = self.original_env.take() {
            for (key, _) in self.new_env.take().unwrap() {
                std::env::remove_var(key);
            }
            for (key, value) in original_env {
                std::env::set_var(key, value);
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct TestBuilder {
    with_env: Option<Vec<(String, String)>>,
    unset_env: bool,
    with_temp_state_file: bool,
    with_temp_settings_file: bool,
}

impl TestBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set environment variables for the test.
    /// If `unset` is true, all environment variables will be unset before setting the new ones.
    /// All environment variables will be restored after the test.
    pub fn with_env(
        mut self,
        env: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>,
        unset: bool,
    ) -> Self {
        self.with_env = Some(env.into_iter().map(|(k, v)| (k.into(), v.into())).collect());
        self.unset_env = unset;
        self
    }

    pub fn with_temp_state_file(mut self) -> Self {
        self.with_temp_state_file = true;
        self
    }

    pub fn with_temp_settings_file(mut self) -> Self {
        self.with_temp_settings_file = true;
        self
    }

    pub fn execute_sync<F: FnOnce() -> R, R>(self, f: F) -> R {
        let _lock = ENV_LOCK.blocking_lock();

        let mut data_store = DataStore::default();
        if let Some(env) = self.with_env {
            data_store = data_store.with_env(env, self.unset_env);
        }

        if self.with_temp_settings_file {
            let file = tempfile::tempfile().unwrap();
            *TEMP_SETTINGS_FILE.blocking_lock() = Some(file);
        }

        if self.with_temp_state_file {
            let file = tempfile::tempfile().unwrap();
            *TEMP_STATE_FILE.blocking_lock() = Some(file);
        }

        let result = f();

        data_store.restore();

        TEMP_SETTINGS_FILE.blocking_lock().take();
        TEMP_STATE_FILE.blocking_lock().take();

        result
    }

    pub async fn execute<F: FnOnce() -> Fut, Fut: Future<Output = R>, R>(self, f: F) -> R {
        let _lock = ENV_LOCK.lock().await;

        let mut data_store = DataStore::default();
        if let Some(env) = self.with_env {
            data_store = data_store.with_env(env, self.unset_env);
        }

        if self.with_temp_settings_file {
            let file = tempfile::tempfile().unwrap();
            *TEMP_SETTINGS_FILE.lock().await = Some(file);
        }

        if self.with_temp_state_file {
            let file = tempfile::tempfile().unwrap();
            *TEMP_STATE_FILE.lock().await = Some(file);
        }

        let result = f().await;

        data_store.restore();

        TEMP_SETTINGS_FILE.lock().await.take();
        TEMP_STATE_FILE.lock().await.take();

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::const_new(());

    #[test]
    fn test_env_sync() {
        let _lock = TEST_LOCK.blocking_lock();

        TestBuilder::new().with_env([("BAR", "1")], false).execute_sync(|| {
            assert_eq!(std::env::var("BAR").unwrap(), "1");
        });

        assert_eq!(std::env::var("BAR").unwrap_err(), std::env::VarError::NotPresent);
    }

    #[tokio::test]
    async fn test_env_async() {
        let _lock = TEST_LOCK.lock().await;

        TestBuilder::new()
            .with_env([("BAR", "1")], false)
            .execute(|| async {
                assert_eq!(std::env::var("BAR").unwrap(), "1");
            })
            .await;

        assert_eq!(std::env::var("BAR").unwrap_err(), std::env::VarError::NotPresent);
    }

    #[test]
    fn test_temp_files_sync() {
        let _lock = TEST_LOCK.blocking_lock();

        TestBuilder::new()
            .with_temp_settings_file()
            .with_temp_state_file()
            .execute_sync(|| {
                assert!(TEMP_SETTINGS_FILE.blocking_lock().is_some());
                assert!(TEMP_STATE_FILE.blocking_lock().is_some());
            });

        assert!(TEMP_SETTINGS_FILE.blocking_lock().is_none());
        assert!(TEMP_STATE_FILE.blocking_lock().is_none());
    }

    #[tokio::test]
    async fn test_temp_files_async() {
        let _lock = TEST_LOCK.lock().await;

        TestBuilder::new()
            .with_temp_settings_file()
            .with_temp_state_file()
            .execute(|| async {
                assert!(TEMP_SETTINGS_FILE.lock().await.is_some());
                assert!(TEMP_STATE_FILE.lock().await.is_some());
            })
            .await;

        assert!(TEMP_SETTINGS_FILE.lock().await.is_none());
        assert!(TEMP_STATE_FILE.lock().await.is_none());
    }

    #[tokio::test]
    async fn stress_test_1() {
        TestBuilder::new()
            .execute(|| async {
                for _ in 0..10000 {
                    std::env::set_var("FIG_TEST_VAR", "1");
                    assert_eq!(std::env::var("FIG_TEST_VAR").unwrap(), "1");
                    std::env::set_var("FIG_TEST_VAR", "2");
                }
            })
            .await;
    }

    #[tokio::test]
    async fn stress_test_2() {
        TestBuilder::new()
            .execute(|| async {
                for _ in 0..10000 {
                    std::env::set_var("FIG_TEST_VAR", "3");
                    assert_eq!(std::env::var("FIG_TEST_VAR").unwrap(), "3");
                    std::env::set_var("FIG_TEST_VAR", "4");
                }
            })
            .await;
    }
}
