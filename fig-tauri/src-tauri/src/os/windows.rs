use std::path::Path;

#[derive(Default)]
pub struct State {
    _window_id: u32,
    _process_id: u32,
}

pub struct Listener(());

impl Listener {
    pub fn bind(path: &Path) -> Self {
        todo!()
    }
}
