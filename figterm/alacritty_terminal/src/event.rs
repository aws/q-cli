use crate::term::{SizeInfo, ShellState};

/// Terminal event.
///
/// These events instruct the TODO socket over changes that can't be handled by the terminal emulation layer
/// itself.
#[derive(Debug, Clone)]
pub enum Event {
    Prompt,
    PreExec,
}

/// Types that are interested in when the display is resized.
pub trait OnResize {
    fn on_resize(&mut self, size: &SizeInfo);
}

/// Event Loop for sending info about terminal events.
pub trait EventListener {
    fn send_event(&self, _event: Event, _shell_state: &ShellState) {}
}

/// Placeholder implementation for tests.
#[cfg(test)]
impl EventListener for () {}
