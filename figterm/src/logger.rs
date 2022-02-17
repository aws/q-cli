//! [log] logger

use anyhow::{Context, Result};
use nix::unistd::getpid;
use std::{
    env,
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::utils::fig_path;

pub fn get_fig_log_level() -> log::LevelFilter {
    match env::var("FIG_LOG_LEVEL")
        .ok()
        .map(|s| log::LevelFilter::from_str(&*s).ok())
        .flatten()
    {
        Some(level) => level,
        _ => log::LevelFilter::Debug,
    }
}

pub fn stdio_debug_log(s: impl AsRef<str>) {
    if get_fig_log_level() >= log::Level::Debug {
        println!("{}", s.as_ref());
    }
}

pub fn init_logger(ptc_name: impl AsRef<str>) -> Result<()> {
    let log_level = get_fig_log_level();
    let logger = Logger::new(&ptc_name)?;
    log::set_boxed_logger(Box::new(logger)).map(|_| log::set_max_level(log_level))?;
    Ok(())
}

#[derive(Debug)]
struct Logger {
    file: Arc<Mutex<File>>,
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

impl Logger {
    fn new(ptc_name: impl AsRef<str>) -> Result<Self> {
        create_dir_all(log_folder()?)?;
        let file = Arc::new(Mutex::new(File::create(log_path(ptc_name)?)?));
        Ok(Self { file })
    }
}

impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            let mut file = self.file.lock().unwrap();
            writeln!(
                file,
                "\x1B[38;5;168mfigterm ({}):\x1B[0m [{}:{}] {}",
                getpid(),
                record.file_static().unwrap_or("?"),
                record
                    .line()
                    .map(|i| i.to_string())
                    .unwrap_or_else(|| "?".into()),
                record.args()
            )
            .ok();
        }
    }

    fn flush(&self) {}
}
