use std::path::PathBuf;

use super::{
    Pid,
    PidExt,
};

impl PidExt for Pid {
    fn current() -> Self {
        nix::unistd::getpid().into()
    }

    fn parent(&self) -> Option<Pid> {
        let mut buffer = [0u32; 1];
        let buffer_ptr = buffer.as_mut_ptr() as *mut c_void;
        let ret = unsafe { proc_listpids(6, 0, buffer_ptr, 1) };
        if ret <= 0 { None } else { buffer[0].into() }
    }

    fn exe(&self) -> Option<PathBuf> {
        let mut buffer = [0u8; 4096];
        let buffer_ptr = buffer.as_mut_ptr() as *mut std::ffi::c_void;
        let buffer_size = buffer.len() as u32;
        let ret = unsafe { nix::libc::proc_pidpath(self, buffer_ptr, buffer_size) };
        if ret <= 0 { None } else { PathBuf::from(buffer[..ret]) }
    }
}
