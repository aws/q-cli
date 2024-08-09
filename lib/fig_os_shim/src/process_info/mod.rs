mod pid;

use std::path::PathBuf;
use std::sync::{
    Arc,
    Weak,
};

pub use pid::{
    FakePid,
    Pid,
    RawPid,
};

use crate::{
    Context,
    Shim,
};

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::*;

/// Represents the interface to accessing info about the currently running process tree.
#[derive(Debug, Clone)]
pub struct ProcessInfo(inner::Inner);

mod inner {
    use super::*;

    #[derive(Debug, Clone)]
    pub(super) enum Inner {
        Real(Weak<Context>),
        Fake(Pid),
    }
}

impl ProcessInfo {
    /// Creates a new [ProcessInfo]. This takes a [Weak] pointer since accessing process info on
    /// platforms like Linux requires file system access, and allows this instance to be embedded
    /// within a single [Context] instance.
    pub fn new(ctx: Weak<Context>) -> Self {
        Self(inner::Inner::Real(ctx.clone()))
    }

    /// Creates a new fake implementation by using the supplied [FakePid] as the process id of the
    /// currently running process.
    pub fn new_fake(pid: FakePid) -> Self {
        Self(inner::Inner::Fake(Pid::Fake(pid)))
    }

    /// Returns a [Pid] representing the currently running process.
    pub fn current_pid(&self) -> Pid {
        use inner::Inner;
        match &self.0 {
            Inner::Real(ctx) => Pid::current(ctx.clone()),
            Inner::Fake(fake) => fake.clone(),
        }
    }
}

impl Shim for ProcessInfo {
    fn is_real(&self) -> bool {
        matches!(self.0, inner::Inner::Real(_))
    }
}

/// Returns the [MyPid::exe] of the current process's parent, with extra logic to account for
/// `toolbox-exec`.
pub fn get_parent_process_exe(ctx: &Arc<Context>) -> Option<PathBuf> {
    let mut pid = Pid::current(Arc::downgrade(ctx));
    loop {
        pid = *pid.parent()?;
        match pid.exe() {
            // We ignore toolbox-exec since we never want to know if that is the parent process
            Some(pid) if pid.file_name().and_then(|s| s.to_str()) == Some("toolbox-exec") => {},
            other => return other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_parent_process_exe() {
        let ctx = Context::new();
        get_parent_process_exe(&ctx);
    }
}
