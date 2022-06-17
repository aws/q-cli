//! Figterm Protocal Buffers

use std::iter::repeat;

mod proto {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/figterm.rs"));
}

pub use proto::*;

impl InsertTextCommand {
    pub fn to_term_string(&self) -> String {
        let mut out = String::new();

        match &self.offset.map(|i| i.signum()) {
            Some(1) => out.extend(repeat("\x1b[C").take(self.offset.unwrap_or(0).unsigned_abs() as usize)),
            Some(-1) => out.extend(repeat("\x1b[D").take(self.offset.unwrap_or(0).unsigned_abs() as usize)),
            _ => {},
        }

        out.extend(repeat('\x08').take(self.deletion.unwrap_or(0) as usize));

        if let Some(insertion) = &self.insertion {
            out.push_str(insertion);
        }

        if self.immediate == Some(true) {
            out.push('\r');
        }

        out
    }
}
