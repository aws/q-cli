use std::os::unix::prelude::*;

use anyhow::Result;
use nix::pty::Winsize;
use nix::sys::termios::{
    ControlFlags,
    InputFlags,
    LocalFlags,
    OutputFlags,
    Termios,
};
use nix::{
    ioctl_read_bad,
    libc,
};

ioctl_read_bad!(read_winsize, libc::TIOCGWINSZ, Winsize);

/// Get the winsize of a [`RawFd`]
pub fn get_winsize(fd: RawFd) -> Result<Winsize> {
    let mut winsize = Winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };

    unsafe { read_winsize(fd, &mut winsize) }?;

    Ok(winsize)
}

/// Convert a [`Termios`] into raw mode
pub fn termios_to_raw(mut termios: Termios) -> Termios {
    // Turn off echo, canonical mode extended input processing & signal chars.
    termios
        .local_flags
        .remove(LocalFlags::ECHO | LocalFlags::ICANON | LocalFlags::IEXTEN | LocalFlags::ISIG);

    // Turn off SIGINT on BREAK, CR-to-NL, input parity check, strip 8th bit on input,
    // and output control flow.
    termios.input_flags.remove(
        InputFlags::BRKINT
            | InputFlags::ICRNL
            | InputFlags::IGNBRK
            | InputFlags::INPCK
            | InputFlags::ISTRIP
            | InputFlags::IXON,
    );

    // Clear size bits, parity checking off.
    termios.control_flags.remove(ControlFlags::CSIZE | ControlFlags::PARENB);

    // 8 bits/char
    termios.control_flags.insert(ControlFlags::CS8);

    // Output processing off.
    termios.output_flags.remove(OutputFlags::OPOST);

    // Set case b, 1 byte at a time, no timer.
    termios.control_chars[libc::VMIN] = 1;
    termios.control_chars[libc::VTIME] = 0;

    termios
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "desktop-tests")]
    fn to_raw_test() {
        use nix::libc::STDIN_FILENO;
        use nix::sys::termios::tcgetattr;

        use super::termios_to_raw;

        let termios = tcgetattr(STDIN_FILENO).unwrap();
        termios_to_raw(termios);
    }
}
