#[derive(Default)]
pub struct State {
    _window_id: u32,
    _process_id: u32,
}

// NOTE: whatever this returns has to implement GenericSocket
pub fn bind_socket(_path: &PathBuf) {
    todo!()
}
