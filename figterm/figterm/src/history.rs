//! Fig history logging

use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
};

use anyhow::Result;

use crate::utils::fig_path;

#[derive(Debug)]
pub struct HistoryFile {
    file: File,
}

impl HistoryFile {
    pub fn new() -> Result<HistoryFile> {
        let mut path = fig_path().unwrap();
        path.push("history2");

        let file = OpenOptions::new().append(true).create(true).open(path)?;

        Ok(HistoryFile { file })
    }

    pub fn write_entry(&mut self, entry: &HistoryEntry) -> Result<()> {
        write!(self.file, "{}", entry.to_history_file_string())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct HistoryEntry {
    command: String,
    shell: String,
    pid: u64,
    session_id: String,
    cwd: PathBuf,
    time: u64,

    in_ssh: bool,
    in_docker: bool,
    hostname: Option<String>,

    exit_code: u32,
}

impl HistoryEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        command: String,
        shell: String,
        pid: u64,
        session_id: String,
        cwd: PathBuf,
        time: u64,

        in_ssh: bool,
        in_docker: bool,
        hostname: Option<String>,

        exit_code: u32,
    ) -> HistoryEntry {
        HistoryEntry {
            command,
            shell,
            pid,
            session_id,
            cwd,
            time,
            in_ssh,
            in_docker,
            hostname,
            exit_code,
        }
    }

    fn to_history_file_string(&self) -> String {
        let mut string = String::new();

        string.push_str(&format!("\n- command: {}", self.command));
        string.push_str(&format!("\n  exit_code: {}", self.exit_code));
        string.push_str(&format!("\n  shell: {}", self.shell));
        string.push_str(&format!("\n  session_id: {}", self.session_id));
        string.push_str(&format!("\n  cwd: {}", self.cwd.to_string_lossy()));
        string.push_str(&format!("\n  time: {}", self.time));

        if self.in_ssh {
            string.push_str("\n  ssh: true");
        }

        if self.in_docker {
            string.push_str("\n  docker: true");
        }

        if let Some(hostname) = &self.hostname {
            string.push_str(&format!("\n  hostname: {}", hostname));
        }

        string
    }
}
