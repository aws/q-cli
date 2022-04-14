use std::path::Path;

mod ipc;
mod uiautomation;

pub const SHELL: &str = "wsl";
pub const SHELL_ARGS: [&str; 0] = [];

#[derive(Default, Debug)]
pub struct State {
    _window_id: u32,
    _process_id: u32,
}

pub struct Listener(ipc::WindowsListener);

impl Listener {
    pub fn bind(path: &Path) -> Self {
        Self(ipc::WindowsListener::bind(path).expect("Failed to bind to socket"))
    }

    pub async fn accept(&self) -> Result<ipc::WindowsStream, ipc::WindowsSocketError> {
        self.0.accept().await
    }
}

pub fn init() {
    std::thread::spawn(uiautomation::ui_listener_event_loop);
}
