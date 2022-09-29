use super::window_position::WindowPosition;

#[derive(Debug)]
pub struct ActiveWindow {
    pub window_id: String,
    // Or pass complete application object???
    pub process_id: u64,
    pub position: WindowPosition,
    pub bundle_id: String,
}
