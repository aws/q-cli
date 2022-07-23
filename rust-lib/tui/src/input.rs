use newton::{
    KeyCode,
    KeyModifiers,
};

#[non_exhaustive]
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
    Exit,
    Reenter,
    Insert(char, KeyModifiers),
}

impl InputAction {
    pub(crate) fn from_key(input_method: &InputMethod, code: KeyCode, modifiers: KeyModifiers) -> Vec<Self> {
        match input_method {
            InputMethod::None => match code {
                KeyCode::Char(c) => match c {
                    c if ['o', 'O'].contains(&c) && modifiers.contains(KeyModifiers::CONTROL) => {
                        vec![InputAction::Reenter]
                    },
                    c if ['c', 'C', 'd', 'D'].contains(&c) && modifiers.contains(KeyModifiers::CONTROL) => {
                        vec![InputAction::Exit]
                    },
                    _ => vec![],
                },
                KeyCode::Esc => vec![InputAction::Exit],
                _ => vec![],
            },
            InputMethod::Form => match code {
                KeyCode::Backspace => vec![InputAction::Remove],
                KeyCode::Enter => vec![InputAction::Submit],
                KeyCode::Left => vec![InputAction::Left],
                KeyCode::Right => vec![InputAction::Right],
                KeyCode::Up => vec![InputAction::Up],
                KeyCode::Down => vec![InputAction::Down],
                KeyCode::Tab => vec![InputAction::Next],
                KeyCode::BackTab => vec![InputAction::Previous],
                KeyCode::Delete => vec![InputAction::Delete],
                KeyCode::Char(c) => match c {
                    c if c == ' ' => vec![InputAction::Select, InputAction::Insert(c, modifiers)],
                    c if ['o', 'O'].contains(&c) && modifiers.contains(KeyModifiers::CONTROL) => {
                        vec![InputAction::Reenter]
                    },
                    c if ['c', 'C', 'd', 'D'].contains(&c) && modifiers.contains(KeyModifiers::CONTROL) => {
                        vec![InputAction::Exit]
                    },
                    _ => vec![InputAction::Insert(c, modifiers)],
                },
                KeyCode::Esc => vec![InputAction::Exit],
                _ => vec![],
            },
        }
    }
}

#[non_exhaustive]
pub enum InputMethod {
    None,
    Form,
}
