use core_graphics::display::CGRect;

use crate::macos::general::window_position::WindowPosition;

pub trait FromCgRect {
    fn from_cg_rect(cgrect: &CGRect) -> WindowPosition;
}

impl FromCgRect for WindowPosition {
    fn from_cg_rect(cgrect: &CGRect) -> Self {
        Self::new(cgrect.origin.x, cgrect.origin.y, cgrect.size.width, cgrect.size.height)
    }
}
