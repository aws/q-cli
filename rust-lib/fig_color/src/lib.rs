//! A safe wrapper for the old figterm `color.c`

mod color;

use std::fmt::Debug;

use bitflags::bitflags;

bitflags! {
    pub struct ColorSupport: u32 {
        const TERM256   = 0b0000_0000_0000_0000_0001;
        const TERM24BIT = 0b0000_0000_0000_0000_0010;
    }
}

impl From<ColorSupport> for color::color_support_t {
    fn from(val: ColorSupport) -> Self {
        val.bits
    }
}

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

#[derive(Clone)]
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
    let color_support = color::get_color_support();
    ColorSupport::from_bits_truncate(color_support)
}

pub fn parse_suggestion_color_fish(suggestion_str: &str, color_support: ColorSupport) -> Option<SuggestionColor> {
    let inner = color::parse_suggestion_color_fish(suggestion_str, color_support.into());
    inner.map(|inner| SuggestionColor { inner })
}

pub fn parse_suggestion_color_zsh_autosuggest(
    suggestion_str: &str,
    color_support: ColorSupport,
) -> Option<SuggestionColor> {
    let inner = color::parse_suggestion_color_zsh_autosuggest(suggestion_str, color_support.into());
    inner.map(|inner| SuggestionColor { inner })
}
