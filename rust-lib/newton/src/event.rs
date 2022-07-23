pub use crossterm::event::{
    KeyCode,
    KeyModifiers,
};
use crossterm::event::{
    MouseButton,
    MouseEvent,
    MouseEventKind,
};

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Initialize { width: i32, height: i32 },
    Update,
    Draw,
    Resized { width: i32, height: i32 },
    KeyPressed { code: KeyCode, modifiers: KeyModifiers },
    MouseMoved { column: i32, row: i32 },
    MousePressed { button: MouseButton, column: i32, row: i32 },
    MouseReleased { button: MouseButton, column: i32, row: i32 },
    MouseScrollUp { column: i32, row: i32 },
    MouseScrollDown { column: i32, row: i32 },
}

impl From<crossterm::event::Event> for Event {
    fn from(from: crossterm::event::Event) -> Self {
        use crossterm::event::Event::*;

        match from {
            Key(key) => Event::KeyPressed {
                code: key.code,
                modifiers: key.modifiers,
            },
            Mouse(MouseEvent { kind, column, row, .. }) => match kind {
                MouseEventKind::Down(button) => Event::MousePressed {
                    button,
                    column: column.into(),
                    row: row.into(),
                },
                MouseEventKind::Up(button) => Event::MouseReleased {
                    button,
                    column: column.into(),
                    row: row.into(),
                },
                MouseEventKind::Drag(_) => Event::MouseMoved {
                    column: column.into(),
                    row: row.into(),
                },
                MouseEventKind::Moved => Event::MouseMoved {
                    column: column.into(),
                    row: row.into(),
                },
                MouseEventKind::ScrollDown => Event::MouseScrollDown {
                    column: column.into(),
                    row: row.into(),
                },
                MouseEventKind::ScrollUp => Event::MouseScrollUp {
                    column: column.into(),
                    row: row.into(),
                },
            },
            Resize(width, height) => Event::Resized {
                width: width.into(),
                height: height.into(),
            },
        }
    }
}
