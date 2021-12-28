use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CommandInfo {
    pub command: String,
    pub shell: Option<String>,
    pub pid: Option<i32>,
    pub session_id: Option<String>,
    pub cwd: Option<PathBuf>,
    pub time: u64,

    pub hostname: Option<String>,
    pub in_ssh: bool,
    pub in_docker: bool,

    pub exit_code: Option<i32>,
}
