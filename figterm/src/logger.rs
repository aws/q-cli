//! [log] logger

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use std::{
    fs::{self, File},
    path::PathBuf,
    str::FromStr,
};
use tracing::{level_filters::LevelFilter, Level};
use tracing_subscriber::{filter::DynFilterFn, fmt, prelude::*};

use crate::utils::fig_path;

static FIG_LOG_LEVEL: Lazy<RwLock<LevelFilter>> = Lazy::new(|| RwLock::new(LevelFilter::INFO));

pub fn stdio_debug_log(s: impl AsRef<str>) {
    let level = FIG_LOG_LEVEL.read();
    if *level >= Level::DEBUG {
        println!("{}", s.as_ref());
    }
}

fn log_folder() -> Result<PathBuf> {
    let mut dir = fig_path().context("failed to get fig path")?;
    dir.push("logs");
    Ok(dir)
}

/// Get the path to the pt logfile
fn log_path(ptc_name: impl AsRef<str>) -> Result<PathBuf> {
    let log_file_name = format!("figterm{}.log", ptc_name.as_ref().replace('/', "_"));

    let mut dir = log_folder()?;
    dir.push(log_file_name);
    Ok(dir)
}

pub fn set_log_level(level: LevelFilter) {
    *FIG_LOG_LEVEL.write() = level;
}

pub fn get_log_level() -> LevelFilter {
    *FIG_LOG_LEVEL.read()
}

pub fn init_logger(ptc_name: impl AsRef<str>) -> Result<()> {
    let env_level = std::env::var("FIG_LOG_LEVEL")
        .ok()
        .map(|level| LevelFilter::from_str(&level).ok())
        .flatten()
        .unwrap_or(LevelFilter::INFO);

    *FIG_LOG_LEVEL.write() = env_level;

    let filter_layer =
        DynFilterFn::new(|metadata, _ctx| metadata.level() <= &*FIG_LOG_LEVEL.read());

    let log_path = log_path(ptc_name)?;

    // Make folder if it doesn't exist
    if !log_path.parent().unwrap().exists() {
        stdio_debug_log(format!(
            "Creating log folder: {:?}",
            log_path.parent().unwrap()
        ));
        fs::create_dir_all(log_path.parent().unwrap())?;
    }

    let file = File::create(log_path).context("failed to create log file")?;
    let fmt_layer = fmt::layer().with_target(false).with_writer(file);

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .init();

    Ok(())
}
