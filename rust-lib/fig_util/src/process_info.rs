use std::path::PathBuf;

use sysinfo::{
    get_current_pid,
    ProcessExt,
    System,
    SystemExt,
};

pub fn get_parent_process_exe() -> Option<PathBuf> {
    let sys = System::new_all();
    let pid = get_current_pid().ok()?;
    let process = sys.process(pid)?;
    let ppid = process.parent()?;
    let parent = sys.process(ppid)?;
    Some(parent.exe().to_path_buf())
}

pub fn get_process_parent_name() -> Option<String> {
    let sys = System::new_all();
    let pid = get_current_pid().ok()?;
    let process = sys.process(pid)?;
    let ppid = process.parent()?;
    let parent = sys.process(ppid)?;
    // TODO: Parent.cmd gets the full path without linux limitations, maybe prefer it?
    let shell = parent.name().trim().to_lowercase();
    let shell = shell.strip_prefix('-').unwrap_or(&shell);

    #[cfg(target_os = "window")]
    let shell = shell.strip_suffix(".exe").unwrap_or(&shell);

    Some(shell.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_name() {
        let parent_name = get_process_parent_name();
        assert_eq!(parent_name, Some("cargo".into()));
    }
}
