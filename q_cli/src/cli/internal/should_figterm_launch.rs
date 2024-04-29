use std::borrow::Cow;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::io::{
    stdout,
    Write,
};
use std::process::ExitCode;

#[cfg(any(target_os = "macos", target_os = "linux"))]
#[allow(dead_code)]
struct ProcessInfo {
    pid: fig_util::process_info::Pid,
    name: String,
    is_valid: bool,
    is_special: bool,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
enum Status {
    Launch(Cow<'static, str>),
    DontLaunch(Cow<'static, str>),
    Process(ProcessInfo),
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl Status {
    fn exit_status(self, quiet: bool) -> Result<ProcessInfo, u8> {
        match self {
            Status::Process(info) => Ok(info),
            Status::Launch(s) => {
                if !quiet {
                    writeln!(stdout(), "✅ {s}").ok();
                }
                Err(0)
            },
            Status::DontLaunch(s) => {
                if !quiet {
                    writeln!(stdout(), "❌ {s}").ok();
                }
                Err(1)
            },
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn parent_status(current_pid: fig_util::process_info::Pid) -> Status {
    use fig_util::env_var::Q_TERM;
    use fig_util::process_info::PidExt;

    let parent_pid = match current_pid.parent() {
        Some(pid) => pid,
        None => return Status::DontLaunch("No parent PID".into()),
    };

    let parent_path = match parent_pid.exe() {
        Some(path) => path,
        None => return Status::DontLaunch("No parent path".into()),
    };

    let parent_name = match parent_path.file_name() {
        Some(name) => match name.to_str() {
            Some(name) => name,
            None => return Status::DontLaunch("Parent name is not valid unicode".into()),
        },
        None => return Status::DontLaunch("No parent name".into()),
    };

    let valid_parent = ["zsh", "bash", "fish", "nu"].contains(&parent_name);

    if fig_util::system_info::in_ssh() {
        return match std::env::var_os(Q_TERM) {
            Some(_) => Status::DontLaunch(format!("In SSH and {Q_TERM} is set").into()),
            None => Status::Launch(format!("In SSH and {Q_TERM} is not set").into()),
        };
    }

    if fig_util::system_info::in_codespaces() {
        return match std::env::var_os(Q_TERM) {
            Some(_) => Status::DontLaunch(format!("In Codespaces and {Q_TERM} is set").into()),
            None => Status::Launch(format!("In Codespaces and {Q_TERM} is not set").into()),
        };
    }

    Status::Process(ProcessInfo {
        pid: parent_pid,
        name: parent_name.into(),
        is_valid: valid_parent,
        is_special: false,
    })
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn grandparent_status(parent_pid: fig_util::process_info::Pid) -> Status {
    #[cfg(target_os = "linux")]
    use fig_util::process_info::LinuxExt;
    use fig_util::process_info::PidExt;
    use fig_util::Terminal;

    let Some(grandparent_pid) = parent_pid.parent() else {
        return Status::DontLaunch("No grandparent PID".into());
    };

    let Some(grandparent_path) = grandparent_pid.exe() else {
        return Status::DontLaunch("No grandparent path".into());
    };

    let grandparent_name = match grandparent_path.file_name() {
        Some(name) => match name.to_str() {
            Some(name) => name,
            None => return Status::DontLaunch("Grandparent name is not a valid utf8 str".into()),
        },
        None => return Status::DontLaunch("No grandparent name".into()),
    };

    // The function to check if the grandparent is a terminal that valid

    #[cfg(target_os = "macos")]
    let check_fn = |terminal: &Terminal| terminal.executable_names().contains(&grandparent_name);

    #[cfg(target_os = "linux")]
    let Some(grandparent_cmdline) = grandparent_pid.cmdline() else {
        return Status::DontLaunch("No grandparent cmdline".into());
    };
    #[cfg(target_os = "linux")]
    let Some(grandparent_exe) = grandparent_cmdline.split('/').last() else {
        return Status::DontLaunch("No grandparent exe".into());
    };

    #[cfg(target_os = "linux")]
    let check_fn = |terminal: &Terminal| {
        terminal.executable_names().contains(&grandparent_name)
            || terminal.executable_names().contains(&grandparent_exe)
    };

    // The terminals the platform supports

    #[cfg(target_os = "macos")]
    let terminals = fig_util::terminal::MACOS_TERMINALS;
    #[cfg(target_os = "linux")]
    let terminals = fig_util::terminal::LINUX_TERMINALS;

    let valid_grandparent = terminals
        .iter()
        .chain(fig_util::terminal::SPECIAL_TERMINALS.iter())
        .find(|term| check_fn(term));

    Status::Process(ProcessInfo {
        pid: grandparent_pid,
        name: grandparent_name.into(),
        is_valid: valid_grandparent.is_some(),
        is_special: valid_grandparent.is_some_and(|term| term.is_special()),
    })
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn should_launch(quiet: bool) -> u8 {
    use fig_util::process_info::PidExt;

    let current_pid = fig_util::process_info::Pid::current();
    let parent_info = match parent_status(current_pid).exit_status(quiet) {
        Ok(info) => info,
        Err(i) => return i,
    };
    let grandparent_info = match grandparent_status(parent_info.pid).exit_status(quiet) {
        Ok(info) => info,
        Err(i) => return i,
    };

    if !quiet {
        let ancestry = format!(
            "{} {} ({}) <- {} {} ({})",
            if grandparent_info.is_valid { "✅" } else { "❌" },
            grandparent_info.name,
            grandparent_info.pid,
            if parent_info.is_valid { "✅" } else { "❌" },
            parent_info.name,
            parent_info.pid,
        );

        writeln!(stdout(), "{ancestry}").ok();
    }

    #[cfg(target_os = "macos")]
    {
        if !grandparent_info.is_special {
            if !quiet {
                writeln!(stdout(), "🟡 Falling back to old mechanism since on macOS").ok();
            }
            return 2;
        }
    }

    u8::from(!(grandparent_info.is_valid && parent_info.is_valid))
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn should_figterm_launch_exit_status(quiet: bool) -> u8 {
    use fig_util::env_var::{
        PROCESS_LAUNCHED_BY_Q,
        Q_PARENT,
    };

    if std::env::var_os("Q_FORCE_FIGTERM_LAUNCH").is_some() {
        if !quiet {
            writeln!(stdout(), "✅ Q_FORCE_FIGTERM_LAUNCH").ok();
        }
        return 0;
    }

    if std::env::var_os(PROCESS_LAUNCHED_BY_Q).is_some() {
        if !quiet {
            writeln!(stdout(), "❌ {PROCESS_LAUNCHED_BY_Q}").ok();
        }
        return 1;
    }

    // Check if inside Emacs
    if std::env::var_os("INSIDE_EMACS").is_some() {
        if !quiet {
            writeln!(stdout(), "❌ INSIDE_EMACS").ok();
        }
        return 1;
    }

    // Check for Warp Terminal
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        if term_program == "WarpTerminal" {
            if !quiet {
                writeln!(stdout(), "❌ TERM_PROGRAM = WarpTerminal").ok();
            }
            return 1;
        }
    }

    // Check for SecureCRT
    if let Ok(cf_bundle_identifier) = std::env::var("__CFBundleIdentifier") {
        if cf_bundle_identifier == "com.vandyke.SecureCRT" {
            if !quiet {
                writeln!(stdout(), "❌ __CFBundleIdentifier = com.vandyke.SecureCRT").ok();
            }
            return 1;
        }
    }

    // PWSH var is set when launched by `pwsh -Login`, in which case we don't want to init.
    if std::env::var_os("__PWSH_LOGIN_CHECKED").is_some() {
        if !quiet {
            writeln!(stdout(), "❌ __PWSH_LOGIN_CHECKED").ok();
        }
        return 1;
    }

    // Make sure we're not in CI
    if fig_util::system_info::in_ci() {
        if !quiet {
            writeln!(stdout(), "❌ In CI").ok();
        }
        return 1;
    }

    // If we are in SSH and there is no Q_PARENT dont launch
    if fig_util::system_info::in_ssh() && std::env::var_os(Q_PARENT).is_none() {
        if !quiet {
            writeln!(stdout(), "❌ In SSH without Q_PARENT").ok();
        }
        return 1;
    }

    if fig_util::system_info::in_wsl() {
        if !quiet {
            writeln!(stdout(), "🟡 Falling back to old mechanism since in WSL").ok();
        }
        2
    } else {
        should_launch(quiet)
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn should_figterm_launch() -> ExitCode {
    ExitCode::from(should_figterm_launch_exit_status(false))
}

#[cfg(target_os = "windows")]
pub fn should_qterm_launch() -> ExitCode {
    use std::os::windows::io::AsRawHandle;

    use winapi::um::consoleapi::GetConsoleMode;

    let mut mode = 0;
    let stdin_ok = unsafe { GetConsoleMode(std::io::stdin().as_raw_handle() as *mut _, &mut mode) };
    ExitCode::from(if stdin_ok == 1 { 2 } else { 1 });
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn should_qterm_launch() -> ExitCode {
    ExitCode::from(2);
}
