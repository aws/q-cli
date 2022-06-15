pub mod terminal_input_parser;

use anyhow::Result;
use dashmap::DashMap;
use fig_settings::keybindings::KeyBindings;
pub use terminal_input_parser::parse_code;
use tracing::trace;

use self::terminal_input_parser::{
    key_from_text,
    KeyCode,
    KeyModifiers,
};

#[derive(Debug, Clone, Default)]
pub struct KeyInterceptor {
    intercept_all: bool,
    intercept_bind: bool,

    mappings: DashMap<(KeyCode<'static>, KeyModifiers), String, fnv::FnvBuildHasher>,
}

impl KeyInterceptor {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn load_key_intercepts(&mut self) -> Result<()> {
        let actions = KeyBindings::load_hardcoded();

        for action in actions.0 {
            if let Some(default_bindings) = action.default_bindings {
                for binding in default_bindings {
                    if let Some(binding) = key_from_text(binding) {
                        self.mappings.insert(binding, action.identifier.clone());
                    }
                }
            }
        }

        Ok(())
    }

    pub fn set_intercept_all(&mut self, intercept_all: bool) {
        trace!("Setting intercept all to {}", intercept_all);
        self.intercept_all = intercept_all;
    }

    pub fn set_intercept_bind(&mut self, intercept_bind: bool) {
        trace!("Setting intercept bind to {}", intercept_bind);
        self.intercept_bind = intercept_bind;
    }

    pub fn reset(&mut self) {
        self.intercept_all = false;
        self.intercept_bind = false;
    }

    pub fn intercept_key<'a>(&self, key: KeyCode<'a>, modifiers: &KeyModifiers) -> Option<String> {
        trace!("Intercepting key: {:?} {:?}", key, modifiers);
        let owned_key = key.to_owned();
        if self.intercept_all || self.intercept_bind {
            if let Some(action) = self.mappings.get(&(owned_key, *modifiers)) {
                return Some(action.value().to_string());
            }
        }
        None
    }
}
