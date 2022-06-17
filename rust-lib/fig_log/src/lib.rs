use std::fmt::Display;
use std::fs::{
    self,
    File,
};
use std::path::PathBuf;

use anyhow::{
    Context,
    Result,
};
use fig_directories::fig_dir;
use once_cell::sync::Lazy;
use tracing::level_filters::LevelFilter;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{
    fmt,
    EnvFilter,
};

fn filter_layer() -> EnvFilter {
    EnvFilter::builder()
        .with_default_directive(LevelFilter::ERROR.into())
        .with_env_var("FIG_LOG_LEVEL")
        .from_env_lossy()
}

static FIG_LOG_LEVEL: Lazy<LevelFilter> = Lazy::new(|| filter_layer().max_level_hint().unwrap_or(LevelFilter::ERROR));

pub fn stdio_debug_log(s: impl Display) {
    if *FIG_LOG_LEVEL >= Level::DEBUG {
        println!("{s}");
    }
}

fn log_folder() -> Result<PathBuf> {
    let mut dir = fig_dir().context("failed to get fig path")?;
    dir.push("logs");
    Ok(dir)
}

fn log_path(log_file_name: impl AsRef<str>) -> Result<PathBuf> {
    let mut dir = log_folder()?;
    dir.push(log_file_name.as_ref().replace('/', "_").replace('\\', "_"));
    Ok(dir)
}

#[must_use]
pub struct LoggerGuard<const N: usize> {
    _file_guard: Option<WorkerGuard>,
    _stdout_guard: Option<WorkerGuard>,
}

#[derive(Debug, Default)]
pub struct Logger {
    log_file_name: Option<String>,
    stdout_logger: bool,
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

    pub fn init(self) -> Result<LoggerGuard<2>> {
        let filter_layer = filter_layer();
        let registry = tracing_subscriber::registry();

        #[cfg(feature = "console")]
        let registry = registry.with(console_subscriber::spawn());

        let registry = registry.with(filter_layer);

        let (file_layer, _file_guard) = match self.log_file_name {
            Some(log_file_name) => {
                let log_path = log_path(log_file_name)?;

                // Make folder if it doesn't exist
                if !log_path.parent().unwrap().exists() {
                    stdio_debug_log(format!("Creating log folder: {:?}", log_path.parent().unwrap()));
                    fs::create_dir_all(log_path.parent().unwrap())?;
                }

                let file = File::create(log_path).context("failed to create log file")?;
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

        registry.init();

        Ok(LoggerGuard {
            _file_guard,
            _stdout_guard,
        })
    }
}
