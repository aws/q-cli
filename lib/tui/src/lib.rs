#![allow(clippy::too_many_arguments)]

#[doc(hidden)]
pub extern crate paste;

pub mod component;

mod buffered_terminal;
mod event_loop;
mod input;
mod style;
mod style_sheet_ext;
mod surface_ext;

pub use component::Component;
pub use event_loop::{
    ControlFlow,
    DisplayMode,
    Event,
    EventLoop,
    State,
};
pub use input::InputMethod;
pub use lightningcss::stylesheet::{
    ParserOptions,
    StyleSheet,
};
pub use style::{
    BorderStyle,
    Display,
    Property,
    Style,
};
pub use surface_ext::SurfaceExt;
pub use termwiz::cell::Intensity;
pub use termwiz::color::ColorAttribute;
pub use termwiz::surface::Surface;
pub use termwiz::Error;
