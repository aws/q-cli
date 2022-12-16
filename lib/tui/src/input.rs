use termwiz::input::{
    InputEvent,
    KeyCode,
    Modifiers,
};

#[non_exhaustive]
#[derive(Debug)]
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
    ChangeView,
    Insert(char, Modifiers),
}

impl InputAction {
    pub(crate) fn from_key(input_method: &InputMethod, code: KeyCode, modifiers: Modifiers) -> Vec<Self> {
        match input_method {
            InputMethod::None => match code {
                KeyCode::Char(c) => match c {
                    c if ['c', 'C', 'd', 'D'].contains(&c) && modifiers.contains(Modifiers::CTRL) => {
                        vec![InputAction::Terminate]
                    },
                    _ => vec![],
                },
                KeyCode::Escape => vec![InputAction::Terminate],
                _ => vec![],
            },
            InputMethod::ExitAny => vec![InputAction::Quit],
            InputMethod::Form | InputMethod::Scripted(_) => match code {
                KeyCode::Backspace => vec![InputAction::Remove],
                KeyCode::Enter => match modifiers.contains(Modifiers::SHIFT) {
                    true => vec![InputAction::Submit, InputAction::Quit],
                    false => vec![InputAction::Submit],
                },
                KeyCode::LeftArrow => vec![InputAction::Left],
                KeyCode::RightArrow => vec![InputAction::Right],
                KeyCode::UpArrow => vec![InputAction::Up],
                KeyCode::DownArrow => vec![InputAction::Down],
                KeyCode::Tab => match modifiers.contains(Modifiers::SHIFT) {
                    true => vec![InputAction::Previous],
                    false => vec![InputAction::Next],
                },
                KeyCode::Delete => vec![InputAction::Delete],
                KeyCode::Char(c) => match c {
                    c if c == ' ' => vec![InputAction::Select, InputAction::Insert(c, modifiers)],
                    c if ['c', 'C', 'd', 'D'].contains(&c) && modifiers.contains(Modifiers::CTRL) => {
                        vec![InputAction::Terminate]
                    },
                    c if ['o', 'O'].contains(&c) && modifiers.contains(Modifiers::CTRL) => {
                        vec![InputAction::ChangeView]
                    },
                    _ => vec![InputAction::Insert(c, modifiers)],
                },
                KeyCode::Escape => vec![InputAction::Terminate],
                _ => vec![],
            },
        }
    }
}

#[non_exhaustive]
pub enum InputMethod {
    None,
    ExitAny,
    Form,
    Scripted(Vec<InputEvent>),
}
