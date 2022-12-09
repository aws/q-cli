#![allow(clippy::too_many_arguments)]

#[doc(hidden)]
pub extern crate paste;

pub mod component;

mod event_loop;
mod input;
#[macro_use]
mod stylesheet;
mod style;
mod surface_ext;

pub use component::Component;
pub use event_loop::{
    ControlFlow,
    Event,
    EventLoop,
    State,
};
pub use input::InputMethod;
pub use style::{
    BorderStyle,
    Property,
    Style,
};
pub use stylesheet::StyleSheet;
pub use surface_ext::SurfaceExt;
pub use termwiz::cell::Intensity;
pub use termwiz::color::ColorAttribute;
pub use termwiz::surface::Surface;
pub use termwiz::Error;
