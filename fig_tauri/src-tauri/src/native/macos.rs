pub const SHELL: &str = "/bin/bash";
pub const SHELL_ARGS: [&str; 3] = ["--noprofile", "--norc", "-c"];

#[derive(Debug)]
pub struct NativeState;

impl NativeState {
    pub fn new(window_event_sender: UnboundedSender<WindowEvent>) -> Self {
        Self
    }
}
