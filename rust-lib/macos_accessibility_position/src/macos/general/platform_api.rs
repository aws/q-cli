use super::active_window::ActiveWindow;
use super::window_position::WindowPosition;

pub trait PlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ()>;
    fn get_active_window(&self) -> Result<ActiveWindow, ()>;
}
