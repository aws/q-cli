pub extern crate paste;

#[macro_use]
mod style;

pub mod components;
pub mod layouts;

mod component;
mod event;
mod event_loop;
mod stylesheet;

pub use component::Component;
pub use event::{
    Event,
    KeyCode,
    KeyModifiers,
};
pub use event_loop::{
    ControlFlow,
    DisplayMode,
    EventLoop,
};
pub use newton::Color;
pub use style::{
    BorderStyle,
    Style,
    StyleContext,
};
pub use stylesheet::{
    PseudoClass,
    PseudoElement,
    StyleSheet,
};
