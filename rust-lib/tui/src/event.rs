pub use newton::{
    KeyCode,
    KeyModifiers,
};

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Initialize,
    Update { delta_time: f32 },
    Draw { x: u16, y: u16, width: u16, height: u16 },
    KeyPressed { code: KeyCode, modifiers: KeyModifiers },
}
