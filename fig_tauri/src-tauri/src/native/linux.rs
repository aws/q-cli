use tokio::sync::mpsc::UnboundedSender;

use crate::window::WindowEvent;

#[derive(Debug)]
pub struct NativeState;

impl NativeState {
    pub fn new(window_event_sender: UnboundedSender<WindowEvent>) -> Self {
        Self
    }
}
