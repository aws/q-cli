use std::path::PathBuf;

use super::{
    Pid,
    PidExt,
};

impl PidExt for Pid {
    fn current() -> Self {
        todo!()
    }

    fn parent(&self) -> Option<Pid> {
        todo!()
    }

    fn exe(&self) -> Option<PathBuf> {
        todo!()
    }
}
