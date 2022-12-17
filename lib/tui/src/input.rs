use indexmap::IndexMap;
use termwiz::input::{
    KeyCode,
    KeyEvent,
    Modifiers,
};

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum InputAction {
    Remove,
    Submit,
    Left,
    Right,
    Up,
    Down,
    Next,
    Previous,
    Delete,
    Select,
    Quit,
    Terminate,
    Insert(char),
    Paste(String),
    Unbound(KeyCode),
    // todo(chay): this uses a string but should really be generic
    Custom(String),
    // todo(chay): remove this workaround
    TempChangeView,
}

#[derive(Debug, Default)]
pub struct InputMethod {
    map: IndexMap<(KeyCode, Modifiers), InputAction>,
    exit_any: bool,
}

impl InputMethod {
    pub fn new_minimal() -> Self {
        Self {
            map: IndexMap::from([
                ((KeyCode::Char('c'), Modifiers::CTRL), InputAction::Terminate),
                ((KeyCode::Char('C'), Modifiers::CTRL), InputAction::Terminate),
                ((KeyCode::Char('d'), Modifiers::CTRL), InputAction::Terminate),
                ((KeyCode::Char('D'), Modifiers::CTRL), InputAction::Terminate),
                ((KeyCode::Escape, Modifiers::NONE), InputAction::Terminate),
            ]),
            exit_any: false,
        }
    }

    pub fn new_exit_any() -> Self {
        Self {
            map: IndexMap::default(),
            exit_any: true,
        }
    }

    pub fn new() -> Self {
        Self::new_minimal().insert_all([
            ((KeyCode::Backspace, Modifiers::NONE), InputAction::Remove),
            ((KeyCode::Enter, Modifiers::NONE), InputAction::Submit),
            ((KeyCode::LeftArrow, Modifiers::NONE), InputAction::Left),
            ((KeyCode::RightArrow, Modifiers::NONE), InputAction::Right),
            ((KeyCode::UpArrow, Modifiers::NONE), InputAction::Up),
            ((KeyCode::DownArrow, Modifiers::NONE), InputAction::Down),
            ((KeyCode::Tab, Modifiers::SHIFT), InputAction::Previous),
            ((KeyCode::Tab, Modifiers::NONE), InputAction::Next),
            ((KeyCode::Delete, Modifiers::NONE), InputAction::Delete),
            ((KeyCode::Char('o'), Modifiers::CTRL), InputAction::TempChangeView),
            ((KeyCode::Char('O'), Modifiers::CTRL), InputAction::TempChangeView),
        ])
    }

    pub fn insert(mut self, mapping: ((KeyCode, Modifiers), InputAction)) -> Self {
        self.map.insert(mapping.0, mapping.1);
        self
    }

    pub fn insert_all<const N: usize>(mut self, mappings: [((KeyCode, Modifiers), InputAction); N]) -> Self {
        for mapping in mappings {
            self.map.insert(mapping.0, mapping.1);
        }

        self
    }

    pub fn get_action(&self, key_event: KeyEvent) -> InputAction {
        let key = key_event.key;
        let modifiers = key_event.modifiers;

        match self.map.get(&(key, modifiers)) {
            Some(action) => action.clone(),
            None => {
                if self.exit_any {
                    return InputAction::Quit;
                }

                if let KeyCode::Char(key) = key {
                    return InputAction::Insert(key);
                }

                InputAction::Unbound(key)
            },
        }
    }
}
