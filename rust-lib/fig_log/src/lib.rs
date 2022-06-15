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

pub fn stdio_debug_log(s: impl AsRef<str>) {
    if *FIG_LOG_LEVEL >= Level::DEBUG {
        println!("{}", s.as_ref());
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
    _guards: [WorkerGuard; N],
}

pub fn init_logger(log_file_name: impl AsRef<str>) -> Result<LoggerGuard<2>> {
    let filter_layer = filter_layer();
    let log_path = log_path(log_file_name)?;

    // Make folder if it doesn't exist
    if !log_path.parent().unwrap().exists() {
        stdio_debug_log(format!("Creating log folder: {:?}", log_path.parent().unwrap()));
        fs::create_dir_all(log_path.parent().unwrap())?;
    }

    let file = File::create(log_path).context("failed to create log file")?;
    let (non_blocking, guard1) = tracing_appender::non_blocking(file);
    let file_layer = fmt::layer().with_line_number(true).with_writer(non_blocking);

    let (non_blocking, guard2) = tracing_appender::non_blocking(std::io::stdout());
    let stdout_layer = fmt::layer().with_line_number(true).with_writer(non_blocking);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    Ok(LoggerGuard {
        _guards: [guard1, guard2],
    })
}
