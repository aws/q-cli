pub use crossterm::event::{
    KeyCode,
    KeyModifiers,
};

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Initialize { width: u16, height: u16 },
    Update,
    Draw,
    Resized { width: u16, height: u16 },
    KeyPressed { code: KeyCode, modifiers: KeyModifiers },
}

impl From<crossterm::event::Event> for Event {
    fn from(from: crossterm::event::Event) -> Self {
        use crossterm::event::Event::*;

        match from {
            Key(key) => Event::KeyPressed {
                code: key.code,
                modifiers: key.modifiers,
            },
            Mouse(_) => todo!(),
            Resize(width, height) => Event::Resized { width, height },
        }
    }
}
