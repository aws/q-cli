use parking_lot::RwLock;

#[derive(Debug, Default)]
pub struct KeyInterceptState {
    pub intercept_bound_keystrokes: RwLock<bool>,
    pub intercept_global_keystrokes: RwLock<bool>,
}
