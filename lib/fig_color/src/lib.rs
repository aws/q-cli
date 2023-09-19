//! Parsing for terminal color

// todos:
// - More test coverage
// - Combining color.rs and lib.rs
// - Moving this whole crate into alacritty_terminal

mod color;

use std::fmt::Debug;

pub use color::{
    ColorSupport,
    VTermColor,
};

#[derive(Clone, PartialEq, Eq)]
pub struct SuggestionColor {
    inner: color::SuggestionColor,
}

impl SuggestionColor {
    pub fn fg(&self) -> Option<VTermColor> {
        self.inner.fg.clone().map(VTermColor::from)
    }

    pub fn bg(&self) -> Option<VTermColor> {
        self.inner.bg.clone().map(VTermColor::from)
    }
}

impl Debug for SuggestionColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SuggestionColor")
            .field("fg", &self.fg())
            .field("bg", &self.bg())
            .finish()
    }
}

pub fn get_color_support() -> ColorSupport {
    color::get_color_support()
}

pub fn parse_suggestion_color_fish(suggestion_str: &str, color_support: ColorSupport) -> Option<SuggestionColor> {
    let inner = color::parse_suggestion_color_fish(suggestion_str, color_support);
    inner.map(|inner| SuggestionColor { inner })
}

pub fn parse_suggestion_color_zsh_autosuggest(suggestion_str: &str, color_support: ColorSupport) -> SuggestionColor {
    let inner = color::parse_suggestion_color_zsh_autosuggest(suggestion_str, color_support);
    SuggestionColor { inner }
}

pub fn parse_hint_color_nu(suggestion_str: impl AsRef<str>) -> SuggestionColor {
    let color = nu_color_config::lookup_ansi_color_style(suggestion_str.as_ref());
    SuggestionColor {
        inner: color::SuggestionColor {
            fg: color.foreground.map(VTermColor::from),
            bg: color.background.map(VTermColor::from),
        },
    }
}
