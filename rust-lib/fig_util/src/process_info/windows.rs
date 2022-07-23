use std::path::PathBuf;

use windows::Win32::System::Threading::GetCurrentProcessId;

use super::{
    Pid,
    PidExt,
};

impl PidExt for Pid {
    fn current() -> Self {
        unsafe { Pid::from(GetCurrentProcessId()) }
    }

    fn parent(&self) -> Option<Pid> {
        None
    }

    fn exe(&self) -> Option<PathBuf> {
        None
    }
}
