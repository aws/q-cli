use std::borrow::Cow;

use bitflags::bitflags;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KeyCode<'a> {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(Cow<'a, [u8]>),
    Esc,
}

impl<'a> KeyCode<'a> {
    pub fn to_owned(self) -> KeyCode<'static> {
        // Mildly insane that I have to write every variant of
        // this, but the borrow checker forces me to.
        match self {
            KeyCode::Char(c) => KeyCode::Char(c.into_owned().into()),
            KeyCode::F(n) => KeyCode::F(n),
            KeyCode::Backspace => KeyCode::Backspace,
            KeyCode::Enter => KeyCode::Enter,
            KeyCode::Left => KeyCode::Left,
            KeyCode::Right => KeyCode::Right,
            KeyCode::Up => KeyCode::Up,
            KeyCode::Down => KeyCode::Down,
            KeyCode::Home => KeyCode::Home,
            KeyCode::End => KeyCode::End,
            KeyCode::PageUp => KeyCode::PageUp,
            KeyCode::PageDown => KeyCode::PageDown,
            KeyCode::Tab => KeyCode::Tab,
            KeyCode::BackTab => KeyCode::BackTab,
            KeyCode::Delete => KeyCode::Delete,
            KeyCode::Insert => KeyCode::Insert,
            KeyCode::Esc => KeyCode::Esc,
        }
    }
}

bitflags! {
    pub struct KeyModifiers: u8 {
        const SHIFT   = 0b0000_0001;
        const ALT     = 0b0000_0010;
        const CONTROL = 0b0000_0100;
        const META    = 0b0000_1000;
        const NONE    = 0b0000_0000;
    }
}

fn key_modifier_from_u8(modifier: u8) -> Option<KeyModifiers> {
    if modifier == 0 {}
    KeyModifiers::from_bits(modifier - 1)
}

pub fn key_from_text(text: impl AsRef<str>) -> Option<(KeyCode<'static>, KeyModifiers)> {
    let text = text.as_ref();

    let (key_txt, modifier_txt) = match text.split_once('+') {
        Some((modifier, key)) => (key, Some(modifier)),
        None => (text, None),
    };

    let key = match key_txt {
        "backspace" => KeyCode::Backspace,
        "enter" => KeyCode::Enter,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "esc" => KeyCode::Esc,
        f_key if f_key.starts_with("f") => {
            let f_key = f_key.trim_start_matches("f");
            let f_key = f_key.parse::<u8>().ok()?;
            KeyCode::F(f_key)
        }
        c => KeyCode::Char(Cow::Owned(c.as_bytes().to_vec())),
    };

    let modifier = match modifier_txt {
        Some("control") => KeyModifiers::CONTROL,
        Some("shift") => KeyModifiers::SHIFT,
        Some("alt") => KeyModifiers::ALT,
        Some("meta" | "command") => KeyModifiers::META,
        _ => KeyModifiers::NONE,
    };

    Some((key, modifier))
}

pub fn parse_code(code: &[u8]) -> Option<(KeyCode, KeyModifiers)> {
    let mut idx = 0;

    macro_rules! next {
        () => {{
            idx += 1;
            code.get(idx)
        }};
    }

    macro_rules! peek {
        () => {{
            code.get(idx)
        }};
    }

    macro_rules! consume_modifier {
        () => {{
            let mut num = None;
            loop {
                match peek!() {
                    Some(c) if c.is_ascii_digit() => {
                        let digit = (*c - b'0') as u8;
                        match num {
                            None => num = Some(digit),
                            Some(n) => num = Some(n * 10 + digit),
                        }
                        next!();
                    }
                    _ => break,
                }
            }
            num
        }};
    }

    macro_rules! match_vt {
        ($number:expr) => {{
            match $number {
                1 => Some(KeyCode::Home),
                2 => Some(KeyCode::Insert),
                3 => Some(KeyCode::Delete),
                4 => Some(KeyCode::End),
                5 => Some(KeyCode::PageUp),
                6 => Some(KeyCode::PageDown),
                7 => Some(KeyCode::Home),
                8 => Some(KeyCode::End),
                9 => None,
                10 => Some(KeyCode::F(0)),
                11 => Some(KeyCode::F(1)),
                12 => Some(KeyCode::F(2)),
                13 => Some(KeyCode::F(3)),
                14 => Some(KeyCode::F(4)),
                15 => Some(KeyCode::F(5)),
                16 => None,
                17 => Some(KeyCode::F(6)),
                18 => Some(KeyCode::F(7)),
                19 => Some(KeyCode::F(8)),
                20 => Some(KeyCode::F(9)),
                21 => Some(KeyCode::F(10)),
                22 => None,
                23 => Some(KeyCode::F(11)),
                24 => Some(KeyCode::F(12)),
                25 => Some(KeyCode::F(13)),
                26 => Some(KeyCode::F(14)),
                27 => None,
                28 => Some(KeyCode::F(15)),
                29 => Some(KeyCode::F(16)),
                30 => None,
                31 => Some(KeyCode::F(17)),
                32 => Some(KeyCode::F(18)),
                33 => Some(KeyCode::F(19)),
                34 => Some(KeyCode::F(20)),
                _ => None,
            }
        }};
    }

    macro_rules! match_xterm {
        ($char1:expr, $char2:expr) => {{
            match ($char1, $char2) {
                (Some(b'A'), None) => Some(KeyCode::Up),
                (Some(b'B'), None) => Some(KeyCode::Down),
                (Some(b'C'), None) => Some(KeyCode::Right),
                (Some(b'D'), None) => Some(KeyCode::Left),
                (Some(b'E'), None) => None,
                (Some(b'F'), None) => Some(KeyCode::End),
                (Some(b'G'), None) => None,
                (Some(b'H'), None) => Some(KeyCode::Home),
                (Some(b'I'), None) => None,
                (Some(b'J'), None) => None,
                (Some(b'K'), None) => None,
                (Some(b'L'), None) => None,
                (Some(b'M'), None) => None,
                (Some(b'N'), None) => None,
                (Some(b'O'), None) => None,
                (Some(b'1'), Some(b'P')) => Some(KeyCode::F(1)),
                (Some(b'1'), Some(b'Q')) => Some(KeyCode::F(2)),
                (Some(b'1'), Some(b'R')) => Some(KeyCode::F(3)),
                (Some(b'1'), Some(b'S')) => Some(KeyCode::F(4)),
                (Some(b'T'), None) => None,
                (Some(b'U'), None) => None,
                (Some(b'V'), None) => None,
                (Some(b'W'), None) => None,
                (Some(b'X'), None) => None,
                (Some(b'Y'), None) => None,
                (Some(b'Z'), None) => None,
                _ => None,
            }
        }};
    }

    macro_rules! match_ss3 {
        ($char1:expr, $char2:expr) => {{
            match ($char1, $char2) {
                /* One Char */
                /* Uppercase ascii */
                (Some(b'A'), None) => Some(KeyCode::Up),
                (Some(b'B'), None) => Some(KeyCode::Down),
                (Some(b'C'), None) => Some(KeyCode::Right),
                (Some(b'D'), None) => Some(KeyCode::Left),
                (Some(b'E'), None) => None,
                (Some(b'F'), None) => Some(KeyCode::End),
                (Some(b'G'), None) => None,
                (Some(b'H'), None) => Some(KeyCode::Home),
                (Some(b'I'), None) => Some(KeyCode::Tab),
                (Some(b'J'), None) => None,
                (Some(b'K'), None) => None,
                (Some(b'L'), None) => None,
                (Some(b'M'), None) => Some(KeyCode::Enter),
                (Some(b'N'), None) => None,
                (Some(b'O'), None) => None,
                (Some(b'P'), None) => Some(KeyCode::F(1)),
                (Some(b'Q'), None) => Some(KeyCode::F(2)),
                (Some(b'R'), None) => Some(KeyCode::F(3)),
                (Some(b'S'), None) => Some(KeyCode::F(4)),
                (Some(b'T'), None) => None,
                (Some(b'U'), None) => None,
                (Some(b'V'), None) => None,
                (Some(b'W'), None) => None,
                (Some(b'X'), None) => Some(KeyCode::Char(Cow::Borrowed(b"="))),
                (Some(b'Y'), None) => None,
                (Some(b'Z'), None) => None,
                /* Lowercase ascii */
                (Some(b'a'), None) => None,
                (Some(b'b'), None) => None,
                (Some(b'c'), None) => None,
                (Some(b'd'), None) => None,
                (Some(b'e'), None) => None,
                (Some(b'f'), None) => None,
                (Some(b'g'), None) => None,
                (Some(b'h'), None) => None,
                (Some(b'i'), None) => None,
                (Some(b'j'), None) => Some(KeyCode::Char(Cow::Borrowed(b"*"))),
                (Some(b'k'), None) => Some(KeyCode::Char(Cow::Borrowed(b"+"))),
                (Some(b'l'), None) => Some(KeyCode::Char(Cow::Borrowed(b","))),
                (Some(b'm'), None) => Some(KeyCode::Char(Cow::Borrowed(b"-"))),
                (Some(b'n'), None) => Some(KeyCode::Char(Cow::Borrowed(b"."))),
                (Some(b'o'), None) => Some(KeyCode::Char(Cow::Borrowed(b"/"))),
                (Some(b'p'), None) => Some(KeyCode::Char(Cow::Borrowed(b"0"))),
                (Some(b'q'), None) => Some(KeyCode::Char(Cow::Borrowed(b"1"))),
                (Some(b'r'), None) => Some(KeyCode::Char(Cow::Borrowed(b"2"))),
                (Some(b's'), None) => Some(KeyCode::Char(Cow::Borrowed(b"3"))),
                (Some(b't'), None) => Some(KeyCode::Char(Cow::Borrowed(b"4"))),
                (Some(b'u'), None) => Some(KeyCode::Char(Cow::Borrowed(b"5"))),
                (Some(b'v'), None) => Some(KeyCode::Char(Cow::Borrowed(b"6"))),
                (Some(b'w'), None) => Some(KeyCode::Char(Cow::Borrowed(b"7"))),
                (Some(b'x'), None) => Some(KeyCode::Char(Cow::Borrowed(b"8"))),
                (Some(b'y'), None) => Some(KeyCode::Char(Cow::Borrowed(b"9"))),
                (Some(b'z'), None) => None,
                /* Two Char */
                (Some(b'S'), Some(b'P')) => Some(KeyCode::Char(Cow::Borrowed(b" "))),
                _ => None,
            }
        }};
    }

    match peek!() {
        Some(idx_0) => match idx_0 {
            &b'\x1b' => match next!() {
                Some(idx_1) => match idx_1 {
                    &b'\x1b' => Some((KeyCode::Esc, KeyModifiers::NONE)),
                    &b'O' => match_ss3!(next!(), next!()).map(|code| (code, KeyModifiers::NONE)),
                    &b'[' => {
                        next!();
                        let number = consume_modifier!();
                        match (peek!(), number) {
                            /* vt */
                            (Some(b'~' | b';'), Some(number)) => match match_vt!(number) {
                                Some(key) => match peek!() {
                                    Some(b';') => {
                                        next!();
                                        let modifier = consume_modifier!()
                                            .and_then(key_modifier_from_u8)
                                            .unwrap_or(KeyModifiers::NONE);
                                        Some((key, modifier))
                                    }
                                    Some(b'~') => Some((key, KeyModifiers::NONE)),
                                    _ => None,
                                },
                                None => None,
                            },
                            /* xterm */
                            (Some(idx_2), Some(number)) => {
                                next!();
                                let next = next!();
                                match_xterm!(Some(idx_2), next).and_then(|key| {
                                    key_modifier_from_u8(number).map(|modifier| (key, modifier))
                                })
                            }
                            (Some(idx_2), None) => {
                                next!();
                                let next = next!();
                                match_xterm!(Some(idx_2), next).map(|key| (key, KeyModifiers::NONE))
                            }

                            (None, _) => {
                                Some((KeyCode::Char(Cow::Borrowed(b"[")), KeyModifiers::ALT))
                            }
                        }
                    }
                    _ => None,
                },
                None => Some((KeyCode::Esc, KeyModifiers::NONE)),
            },
            _ => match code {
                b"\x08" => Some((KeyCode::Backspace, KeyModifiers::NONE)),
                b"\r" => Some((KeyCode::Enter, KeyModifiers::NONE)),
                b"\x7f" => Some((KeyCode::Delete, KeyModifiers::NONE)),
                b"\t" => Some((KeyCode::Tab, KeyModifiers::NONE)),
                _ => Some((
                    KeyCode::Char(Cow::Borrowed(&code[idx..])),
                    KeyModifiers::NONE,
                )),
            },
        },
        None => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parser() {
        /* <esc> <nochar> -> esc */
        assert_eq!(
            parse_code(b"\x1b"),
            Some((KeyCode::Esc, KeyModifiers::NONE))
        );

        /* <esc> <esc> -> esc */
        assert_eq!(
            parse_code(b"\x1b\x1b"),
            Some((KeyCode::Esc, KeyModifiers::NONE))
        );

        /* <char> -> char */
        assert_eq!(
            parse_code(b"a"),
            Some((KeyCode::Char(Cow::Borrowed(b"a")), KeyModifiers::NONE))
        );

        assert_eq!(
            parse_code(b"hello"),
            Some((KeyCode::Char(Cow::Borrowed(b"hello")), KeyModifiers::NONE))
        );

        /* <esc> '[' <nochar> -> Alt-[ */
        assert_eq!(
            parse_code(b"\x1b["),
            Some((KeyCode::Char(Cow::Borrowed(b"[")), KeyModifiers::ALT))
        );

        /* <esc> '[' (<modifier>) <char> -> keycode sequence, <modifier> is a decimal number and defaults to 1 (xterm) */
        assert_eq!(
            parse_code(b"\x1b[A"),
            Some((KeyCode::Up, KeyModifiers::NONE))
        );

        assert_eq!(
            parse_code(b"\x1b[2A"),
            Some((KeyCode::Up, KeyModifiers::SHIFT))
        );

        assert_eq!(
            parse_code(b"\x1b[3A"),
            Some((KeyCode::Up, KeyModifiers::ALT))
        );

        assert_eq!(
            parse_code(b"\x1b[5A"),
            Some((KeyCode::Up, KeyModifiers::CONTROL))
        );

        assert_eq!(
            parse_code(b"\x1b[9A"),
            Some((KeyCode::Up, KeyModifiers::META))
        );

        assert_eq!(
            parse_code(b"\x1b[13A"),
            Some((KeyCode::Up, KeyModifiers::CONTROL | KeyModifiers::META))
        );

        assert_eq!(
            parse_code(b"\x1b[16A"),
            Some((
                KeyCode::Up,
                KeyModifiers::CONTROL
                    | KeyModifiers::META
                    | KeyModifiers::SHIFT
                    | KeyModifiers::ALT
            ))
        );

        /* <esc> 'O' <char> -> SS3 */
        assert_eq!(
            parse_code(b"\x1bOA"),
            Some((KeyCode::Up, KeyModifiers::NONE))
        );

        assert_eq!(
            parse_code("\x1bOI".as_bytes()),
            Some((KeyCode::Tab, KeyModifiers::NONE))
        );

        assert_eq!(
            parse_code(b"\x1bOSP"),
            Some((KeyCode::Char(Cow::Borrowed(b" ")), KeyModifiers::NONE))
        );

        /* <esc> '[' (<keycode>) (';'<modifier>) '~' -> keycode sequence, <keycode> and <modifier> are decimal numbers and default to 1 (vt) */
        assert_eq!(
            parse_code(b"\x1b[1;5A"),
            Some((KeyCode::Home, KeyModifiers::CONTROL))
        );

        /* <esc> <char> -> Alt-keypress or keycode sequence */
    }
}
