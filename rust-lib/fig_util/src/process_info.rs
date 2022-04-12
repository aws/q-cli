use sysinfo::{get_current_pid, ProcessExt, System, SystemExt};

use crate::Error;

pub fn get_process_parent_name() -> Result<String, Error> {
    let sys = System::new_all();
    let pid = get_current_pid().map_err(|_| Error::UnsupportedPlatform)?;
    let process = sys
        .process(pid)
        .expect("Current pid has no associated process");
    let ppid = process.parent().ok_or(Error::NoParentProcess)?;
    let parent = sys
        .process(ppid)
        .expect("Parent id has no associated process");
    // TODO: Parent.cmd gets the full path without linux limitations, maybe prefer it?
    let shell = parent.name().trim().to_lowercase();
    let shell = shell.strip_prefix('-').unwrap_or(&shell);

    #[cfg(target_os = "window")]
    let shell = shell.strip_suffix(".exe").unwrap_or(&shell);

    Ok(shell.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_name() {
        let parent_name = get_process_parent_name();
        assert!(parent_name.is_ok());
        assert_eq!(parent_name.unwrap(), "cargo");
    }
}
