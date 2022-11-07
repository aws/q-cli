//! Parsing for terminal color

// todos:
// - More test coverage
// - Combining color.rs and lib.rs
// - Moving this whole crate into alacritty_terminal

mod color;

use std::fmt::Debug;

pub use color::ColorSupport;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VTermColor {
    Rgb(u8, u8, u8),
    Indexed(u8),
}

impl From<color::VTermColor> for VTermColor {
    fn from(value: color::VTermColor) -> Self {
        match value {
            color::VTermColor::Rgb { red, green, blue } => Self::Rgb(red, green, blue),
            color::VTermColor::Indexed { idx } => Self::Indexed(idx),
        }
    }
}

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
