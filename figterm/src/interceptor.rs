use anyhow::Result;
use dashmap::DashMap;
use fig_proto::figterm::Action;
use fig_settings::keybindings::{
    KeyBinding,
    KeyBindings,
};
use tracing::trace;

use crate::input::{
    KeyCode,
    KeyEvent,
    Modifiers,
};

pub fn key_from_text(text: impl AsRef<str>) -> Option<KeyEvent> {
    let text = text.as_ref();

    let mut modifiers = Modifiers::NONE;
    let mut remaining = text;
    let key_txt = loop {
        match remaining.split_once('+') {
            Some(("", "")) | None => {
                break remaining;
            },
            Some((modifier_txt, key)) => {
                modifiers |= match modifier_txt {
                    "control" => Modifiers::CTRL,
                    "shift" => Modifiers::SHIFT,
                    "alt" => Modifiers::ALT,
                    "meta" | "command" => Modifiers::META,
                    _ => Modifiers::NONE,
                };
                remaining = key;
            },
        }
    };

    let key = match key_txt {
        "backspace" => KeyCode::Backspace,
        "enter" => KeyCode::Enter,
        "arrowleft" | "left" => KeyCode::LeftArrow,
        "arrowright" | "right" => KeyCode::RightArrow,
        "arrowup" | "up" => KeyCode::UpArrow,
        "arrowdown" | "down" => KeyCode::DownArrow,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        "tab" => KeyCode::Tab,
        // "backtab" => KeyCode::BackTab,
        "delete" => KeyCode::Delete,
        "insert" => KeyCode::Insert,
        "esc" => KeyCode::Escape,
        f_key if f_key.starts_with('f') => {
            let f_key = f_key.trim_start_matches('f');
            let f_key = f_key.parse::<u8>().ok()?;
            KeyCode::Function(f_key)
        },
        c => {
            let mut chars = c.chars();
            let mut first_char = chars.next()?;

            if modifiers.contains(Modifiers::SHIFT) && first_char.is_ascii_lowercase() {
                first_char = first_char.to_ascii_uppercase();
                modifiers.remove(Modifiers::SHIFT);
            }

            if chars.next().is_some() {
                return None;
            }
            KeyCode::Char(first_char)
        },
    };

    Some(KeyEvent { key, modifiers })
}

#[derive(Debug, Clone, Default)]
pub struct KeyInterceptor {
    intercept_all: bool,
    intercept_bind: bool,

    mappings: DashMap<KeyEvent, String, fnv::FnvBuildHasher>,
}

impl KeyInterceptor {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_key_intercepts(&mut self) -> Result<()> {
        let key_bindings = KeyBindings::load_hardcoded();
        for KeyBinding { identifier, binding } in key_bindings {
            if let Some(binding) = key_from_text(binding) {
                self.insert_binding(binding, identifier);
            }
        }
        Ok(())
    }

    pub fn set_intercept_all(&mut self, intercept_all: bool) {
        trace!("Setting intercept all to {intercept_all}");
        self.intercept_all = intercept_all;
    }

    pub fn set_intercept_bind(&mut self, intercept_bind: bool) {
        trace!("Setting intercept bind to {intercept_bind}");
        self.intercept_bind = intercept_bind;
    }

    pub fn set_actions(&mut self, actions: &[Action]) {
        self.mappings.clear();

        for Action { identifier, bindings } in actions {
            for binding in bindings {
                if let Some(binding) = key_from_text(binding) {
                    self.insert_binding(binding, identifier.clone());
                }
            }
        }
    }

    fn insert_binding(&mut self, binding: KeyEvent, identifier: String) {
        if let Some(key) = match binding.key {
            KeyCode::UpArrow => Some(KeyCode::ApplicationUpArrow),
            KeyCode::DownArrow => Some(KeyCode::ApplicationDownArrow),
            KeyCode::LeftArrow => Some(KeyCode::ApplicationLeftArrow),
            KeyCode::RightArrow => Some(KeyCode::ApplicationRightArrow),
            _ => None,
        } {
            self.mappings.insert(
                KeyEvent {
                    key,
                    modifiers: binding.modifiers,
                },
                identifier.clone(),
            );
        };

        if let KeyCode::Char(key) = binding.key {
            // Fill in other case if there is a ctrl or alt, i.e. ctrl+r is the same as ctrl+R
            //
            // This will prevent ctrl+shift+r from being the same as ctrl+r but that is probably
            // fine since we lose context due to parsing ambiguity in the original xterm spec
            // when other modifiers are present
            if (binding.modifiers.contains(Modifiers::CTRL) || binding.modifiers.contains(Modifiers::ALT))
                && key.is_ascii_alphabetic()
            {
                self.mappings.insert(
                    KeyEvent {
                        key: KeyCode::Char(if key.is_ascii_uppercase() {
                            key.to_ascii_lowercase()
                        } else {
                            key.to_ascii_uppercase()
                        }),
                        modifiers: binding.modifiers,
                    },
                    identifier.clone(),
                );
            }
        }

        self.mappings.insert(binding, identifier);
    }

    pub fn reset(&mut self) {
        self.intercept_all = false;
        self.intercept_bind = false;
    }

    pub fn intercept_key(&self, key_event: &KeyEvent) -> Option<String> {
        trace!(?key_event, "Intercepting key");
        if self.intercept_all || self.intercept_bind {
            if let Some(action) = self.mappings.get(key_event) {
                return Some(action.value().to_string());
            }
        }
        None
    }
}
