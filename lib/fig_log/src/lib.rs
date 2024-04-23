use std::fs::{
    self,
    File,
};
use std::path::PathBuf;

use fig_util::directories;
use fig_util::env_var::Q_LOG_LEVEL;
use parking_lot::Mutex;
use thiserror::Error;
use tracing::info;
use tracing::level_filters::LevelFilter;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{
    fmt,
    EnvFilter,
    Registry,
};

const DEFAULT_MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;
const DEFAULT_FILTER: LevelFilter = LevelFilter::ERROR;

static Q_LOG_LEVEL_GLOBAL: Mutex<Option<String>> = Mutex::new(None);
static MAX_LEVEL: Mutex<Option<LevelFilter>> = Mutex::new(None);
static ENV_FILTER_RELOADABLE_HANDLE: Mutex<Option<tracing_subscriber::reload::Handle<EnvFilter, Registry>>> =
    Mutex::new(None);

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Dir(#[from] fig_util::directories::DirectoryError),
    #[error(transparent)]
    TracingReload(#[from] tracing_subscriber::reload::Error),
}

fn log_path(log_file_name: impl AsRef<str>) -> Result<PathBuf> {
    Ok(directories::logs_dir()?.join(log_file_name.as_ref().replace(['/', '\\'], "_")))
}

fn try_fig_log_level() -> Option<String> {
    Q_LOG_LEVEL_GLOBAL
        .lock()
        .clone()
        .or_else(|| std::env::var(Q_LOG_LEVEL).ok())
}

fn fig_log_level() -> String {
    Q_LOG_LEVEL_GLOBAL
        .lock()
        .clone()
        .unwrap_or_else(|| std::env::var(Q_LOG_LEVEL).unwrap_or_else(|_| DEFAULT_FILTER.to_string()))
}

fn create_filter_layer() -> EnvFilter {
    match try_fig_log_level() {
        Some(level) => EnvFilter::builder()
            .with_default_directive(DEFAULT_FILTER.into())
            .parse_lossy(level),
        None => EnvFilter::default().add_directive(Directive::from(DEFAULT_FILTER)),
    }
}

pub fn set_fig_log_level(level: String) -> Result<String> {
    info!("Setting log level to {level:?}");

    let old_level = fig_log_level();
    *Q_LOG_LEVEL_GLOBAL.lock() = Some(level);

    let filter_layer = create_filter_layer();
    *MAX_LEVEL.lock() = filter_layer.max_level_hint();

    ENV_FILTER_RELOADABLE_HANDLE
        .lock()
        .as_ref()
        .expect("set_fig_log_level called before init_logger")
        .reload(filter_layer)?;

    Ok(old_level)
}

pub fn get_fig_log_level() -> String {
    fig_log_level()
}

pub fn get_max_fig_log_level() -> LevelFilter {
    let max_level = *MAX_LEVEL.lock();
    match max_level {
        Some(level) => level,
        None => {
            let filter_layer = create_filter_layer();
            *MAX_LEVEL.lock() = filter_layer.max_level_hint();
            filter_layer.max_level_hint().unwrap_or(DEFAULT_FILTER)
        },
    }
}

#[must_use]
pub struct LoggerGuard {
    _file_guard: Option<WorkerGuard>,
    _stdout_guard: Option<WorkerGuard>,
}

#[derive(Debug, Default)]
pub struct Logger {
    log_file_name: Option<String>,
    stdout_logger: bool,
    max_file_size: Option<u64>,
    delete_old_log_file: bool,
}

impl Logger {
    pub fn new() -> Logger {
        Logger::default()
    }

    pub fn with_stdout(mut self) -> Logger {
        self.stdout_logger = true;
        self
    }

    pub fn with_file(mut self, file_name: impl Into<String>) -> Logger {
        self.log_file_name = Some(file_name.into());
        self
    }

    pub fn with_max_file_size(mut self, size: u64) -> Logger {
        self.max_file_size = Some(size);
        self
    }

    pub fn with_delete_old_log_file(mut self) -> Logger {
        self.delete_old_log_file = true;
        self
    }

    pub fn init(self) -> Result<LoggerGuard> {
        let registry = tracing_subscriber::registry();

        #[cfg(feature = "console")]
        let registry = registry.with(console_subscriber::spawn());

        let filter_layer = create_filter_layer();
        let (reloadable_filter_layer, reloadable_handle) = tracing_subscriber::reload::Layer::new(filter_layer);
        ENV_FILTER_RELOADABLE_HANDLE.lock().replace(reloadable_handle);
        let registry = registry.with(reloadable_filter_layer);

        let (file_layer, _file_guard) = match self.log_file_name {
            Some(log_file_name) => {
                let log_path = log_path(log_file_name)?;

                // Make folder if it doesn't exist
                if let Some(parent) = log_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                if self.delete_old_log_file {
                    fs::remove_file(&log_path).ok();
                } else if log_path.exists() {
                    let metadata = std::fs::metadata(&log_path)?;
                    if metadata.len() > self.max_file_size.unwrap_or(DEFAULT_MAX_FILE_SIZE) {
                        std::fs::remove_file(&log_path)?;
                    }
                }

                let file = if self.delete_old_log_file {
                    File::create(&log_path)?
                } else {
                    File::options().append(true).create(true).open(log_path)?
                };

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = file.metadata() {
                        let mut permissions = metadata.permissions();
                        permissions.set_mode(0o600);
                        file.set_permissions(permissions).ok();
                    }
                }

                let (non_blocking, guard) = tracing_appender::non_blocking(file);
                let file_layer = fmt::layer().with_line_number(true).with_writer(non_blocking);

                (Some(file_layer), Some(guard))
            },
            None => (None, None),
        };
        let registry = registry.with(file_layer);

        let (stdout_layer, _stdout_guard) = if self.stdout_logger {
            let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stdout());
            let stdout_layer = fmt::layer().with_line_number(true).with_writer(non_blocking);
            (Some(stdout_layer), Some(guard))
        } else {
            (None, None)
        };
        let registry = registry.with(stdout_layer);

        // #[cfg(feature = "sentry")]
        // let registry = registry.with(sentry_tracing::layer().event_filter(|md| match md.level() {
        //     &tracing::Level::ERROR | &tracing::Level::WARN => sentry_tracing::EventFilter::Breadcrumb,
        //     _ => sentry_tracing::EventFilter::Ignore,
        // }));

        registry.init();

        Ok(LoggerGuard {
            _file_guard,
            _stdout_guard,
        })
    }
}
