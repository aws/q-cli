//! [log] logger

use anyhow::Result;
use nix::unistd::getpid;
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use crate::utils::fig_path;

pub async fn init_logger(ptc_name: impl AsRef<str>) -> Result<()> {
    let log_level = match std::env::var("FIG_LOG_LEVEL").map(|s| log::LevelFilter::from_str(&*s)) {
        Ok(Ok(level)) => level,
        _ => log::LevelFilter::Trace,
    };

    let logger = Logger::new(&ptc_name)?;
    log::set_boxed_logger(Box::new(logger)).map(|_| log::set_max_level(log_level))?;
    Ok(())
}

#[derive(Debug)]
struct Logger {
    file: Arc<Mutex<File>>,
}

/// Get the path to the pt logfile
fn log_path(ptc_name: impl AsRef<str>) -> Result<PathBuf> {
    let log_file_name = format!("figterm{}.log", ptc_name.as_ref().replace('/', "_"));

    let mut dir = fig_path().unwrap();
    dir.push("logs");
    dir.push(log_file_name);
    Ok(dir)
}

impl Logger {
    fn new(ptc_name: impl AsRef<str>) -> Result<Self> {
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
            .unwrap();
        }
    }

    fn flush(&self) {}
}
