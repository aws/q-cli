//! logger

use std::fs::{
    self,
    File,
};
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{
    Context,
    Result,
};
use fig_directories::fig_dir;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use tracing::level_filters::LevelFilter;
use tracing::Level;
use tracing_subscriber::filter::DynFilterFn;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

static FIG_LOG_LEVEL: Lazy<RwLock<LevelFilter>> = Lazy::new(|| {
    RwLock::new(
        std::env::var("FIG_LOG_LEVEL")
            .ok()
            .and_then(|level| LevelFilter::from_str(&level).ok())
            .unwrap_or(LevelFilter::OFF),
    )
});

pub fn stdio_debug_log(s: impl AsRef<str>) {
    let level = FIG_LOG_LEVEL.read();
    if *level >= Level::DEBUG {
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
    dir.push(log_file_name.as_ref().replace('/', "_"));
    Ok(dir)
}

pub fn set_log_level(level: LevelFilter) {
    *FIG_LOG_LEVEL.write() = level;
}

#[must_use]
pub fn get_log_level() -> LevelFilter {
    *FIG_LOG_LEVEL.read()
}

pub fn init_logger(log_file_name: impl AsRef<str>) -> Result<()> {
    let filter_layer = DynFilterFn::new(|metadata, _ctx| metadata.level() <= &*FIG_LOG_LEVEL.read());

    let log_path = log_path(log_file_name)?;

    // Make folder if it doesn't exist
    if !log_path.parent().unwrap().exists() {
        stdio_debug_log(format!("Creating log folder: {:?}", log_path.parent().unwrap()));
        fs::create_dir_all(log_path.parent().unwrap())?;
    }

    let file = File::create(log_path).context("failed to create log file")?;
    let file_layer = fmt::layer().with_line_number(true).with_writer(file);
    let stdout_layer = fmt::layer().with_line_number(true).with_writer(std::io::stdout);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(file_layer)
        .with(stdout_layer)
        .init();

    Ok(())
}
