#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::io::{
    stdout,
    Write,
};
use std::process::exit;

#[cfg(any(target_os = "macos", target_os = "linux"))]
struct ProcessInfo {
    pid: fig_util::process_info::Pid,
    name: String,
    is_valid: bool,
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
enum Status {
    Launch(String),
    DontLaunch(String),
    Process(ProcessInfo),
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
impl Status {
    fn unwrap(self) -> ProcessInfo {
        match self {
            Status::Process(info) => info,
            Status::Launch(s) => {
                writeln!(stdout(), "✅ {s}").ok();
                exit(0)
            },
            Status::DontLaunch(s) => {
                writeln!(stdout(), "❌ {s}").ok();
                exit(1)
            },
        }
    }
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn parent_status(current_pid: fig_util::process_info::Pid) -> Status {
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
        return match std::env::var_os("FIG_TERM") {
            Some(_) => Status::DontLaunch("In SSH and FIG_TERM is set".into()),
            None => Status::Launch("In SSH and FIG_TERM is not set".into()),
        };
    }

    if fig_util::system_info::in_codespaces() {
        return match std::env::var_os("FIG_TERM") {
            Some(_) => Status::DontLaunch("In Codespaces and FIG_TERM is set".into()),
            None => Status::Launch("In Codespaces and FIG_TERM is not set".into()),
        };
    }

    Status::Process(ProcessInfo {
        pid: parent_pid,
        name: parent_name.into(),
        is_valid: valid_parent,
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
            None => return Status::DontLaunch("Grandparent name is not valid unicode".into()),
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
        .any(check_fn);

    Status::Process(ProcessInfo {
        pid: grandparent_pid,
        name: grandparent_name.into(),
        is_valid: valid_grandparent,
    })
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn should_launch() -> ! {
    use fig_util::process_info::PidExt;

    let current_pid = fig_util::process_info::Pid::current();
    let parent_info = parent_status(current_pid).unwrap();
    let grandparent_info = grandparent_status(parent_info.pid).unwrap();

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

    exit(i32::from(!(grandparent_info.is_valid && parent_info.is_valid)));
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn should_figterm_launch() -> ! {
    if !fig_settings::state::get_bool_or("figterm.enabled", true) {
        writeln!(stdout(), "❌ figterm.enabled is false").ok();
        exit(1);
    }

    if std::env::var_os("PROCESS_LAUNCHED_BY_FIG").is_some() {
        writeln!(stdout(), "❌ PROCESS_LAUNCHED_BY_FIG").ok();
        exit(1);
    }

    // Check if inside Emacs
    if std::env::var_os("INSIDE_EMACS").is_some() {
        writeln!(stdout(), "❌ INSIDE_EMACS").ok();
        exit(1);
    }

    // Check for Warp Terminal
    if let Ok(term_program) = std::env::var("TERM_PROGRAM") {
        if term_program == "WarpTerminal" {
            writeln!(stdout(), "❌ TERM_PROGRAM = WarpTerminal").ok();
            exit(1);
        }
    }

    // Check for SecureCRT
    if let Ok(cf_bundle_identifier) = std::env::var("__CFBundleIdentifier") {
        if cf_bundle_identifier == "com.vandyke.SecureCRT" {
            writeln!(stdout(), "❌ __CFBundleIdentifier = com.vandyke.SecureCRT").ok();
            exit(1);
        }
    }

    // PWSH var is set when launched by `pwsh -Login`, in which case we don't want to init.
    if std::env::var_os("__PWSH_LOGIN_CHECKED").is_some() {
        writeln!(stdout(), "❌ __PWSH_LOGIN_CHECKED").ok();
        exit(1);
    }

    // Make sure we're not in CI
    if fig_util::system_info::in_ci() {
        writeln!(stdout(), "❌ In CI").ok();
        exit(1);
    }

    if std::env::consts::OS == "macos" {
        // For now on macOS we want to fallback to the old mechanism as this is still relatively new
        writeln!(stdout(), "🟡 Falling back to old mechanism since on macOS").ok();
        exit(2);
    } else if fig_util::system_info::in_wsl() {
        writeln!(stdout(), "🟡 Falling back to old mechanism since in WSL").ok();
        exit(2)
    } else {
        should_launch()
    }
}

#[cfg(target_os = "windows")]
pub fn should_figterm_launch() {
    use std::os::windows::io::AsRawHandle;

    use winapi::um::consoleapi::GetConsoleMode;

    let mut mode = 0;
    let stdin_ok = unsafe { GetConsoleMode(std::io::stdin().as_raw_handle() as *mut _, &mut mode) };
    exit(if stdin_ok == 1 { 2 } else { 1 });
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub fn should_figterm_launch() {
    exit(2);
}
