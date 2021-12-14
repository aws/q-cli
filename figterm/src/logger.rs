use anyhow::Result;
use std::{error, path::PathBuf};

use tokio::fs::File;

struct Logger {
    file: File,
}

fn log_path(ptc_name: impl AsRef<str>) -> Result<PathBuf> {
    let log_file_name = format!("figterm{}.log", ptc_name.as_ref().replace('/', "_"));

    let mut dir = dirs::home_dir().unwrap();
    dir.push(".fig");
    dir.push("logs");
    dir.push(log_file_name);
    Ok(dir)
}

impl Logger {
    async fn new(ptc_name: impl AsRef<str>) -> Result<Self, Box<dyn error::Error>> {
        let file = File::open(log_path(ptc_name)?).await?;
        Ok(Self { file })
    }
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::Level::Debug
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!(
                "\033[38;5;168mfigterm ({}):\033[0m [{:?}:{:?}] {}",
                0,
                record.file(),
                record.line(),
                1
            );
        }
    }

    fn flush(&self) {}
}
