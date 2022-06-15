mod display_state;
mod event;
mod event_loop;

pub use crossterm::style::Color;
pub use display_state::DisplayState;
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
