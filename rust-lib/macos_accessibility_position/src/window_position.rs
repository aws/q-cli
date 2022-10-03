use core_graphics::display::CGRect;

#[derive(Debug)]
pub struct WindowPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl WindowPosition {
    pub fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self {
            x,
            y,
            width: w,
            height: h,
        }
    }
}

pub trait FromCgRect {
    fn from_cg_rect(cgrect: &CGRect) -> WindowPosition;
}

impl FromCgRect for WindowPosition {
    fn from_cg_rect(cgrect: &CGRect) -> Self {
        Self::new(cgrect.origin.x, cgrect.origin.y, cgrect.size.width, cgrect.size.height)
    }
}
