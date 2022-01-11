//! A safe wrapper for the old figterm `color.c`

mod color;

use std::{ffi::CString, fmt::Debug};

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

impl TryFrom<color::VTermColor> for VTermColor {
    type Error = u8;

    fn try_from(value: color::VTermColor) -> Result<Self, Self::Error> {
        if unsafe { color::vterm_color_is_indexed(&value as *const _) } {
            Ok(VTermColor::Indexed(unsafe { value.indexed.idx }))
        } else if unsafe { color::vterm_color_is_rgb(&value as *const _) } {
            Ok(VTermColor::Rgb(
                unsafe { value.rgb.red },
                unsafe { value.rgb.green },
                unsafe { value.rgb.blue },
            ))
        } else {
            Err(unsafe { value.type_ })
        }
    }
}

#[derive(Clone)]
pub struct SuggestionColor {
    inner: color::SuggestionColor,
}

impl SuggestionColor {
    pub fn fg(&self) -> Option<VTermColor> {
        match self.inner.fg.is_null() {
            false => VTermColor::try_from(unsafe { *self.inner.fg }).ok(),
            true => None,
        }
    }

    pub fn bg(&self) -> Option<VTermColor> {
        match self.inner.bg.is_null() {
            false => VTermColor::try_from(unsafe { *self.inner.bg }).ok(),
            true => None,
        }
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

impl Drop for SuggestionColor {
    fn drop(&mut self) {
        unsafe { color::free_suggestion_color(&mut self.inner as *mut _) }
    }
}

pub fn get_color_support() -> ColorSupport {
    let color_support = unsafe { color::get_color_support() };
    ColorSupport::from_bits_truncate(color_support)
}

pub fn parse_suggestion_color_fish(
    suggestion_str: impl Into<Vec<u8>>,
    color_support: ColorSupport,
) -> Option<SuggestionColor> {
    let c_str = CString::new(suggestion_str).unwrap();
    let inner = unsafe { color::parse_suggestion_color_fish(c_str.as_ptr(), color_support.into()) };
    match inner.is_null() {
        true => None,
        false => Some(SuggestionColor {
            inner: unsafe { *inner },
        }),
    }
}

pub fn parse_suggestion_color_zsh_autosuggest(
    suggestion_str: impl Into<Vec<u8>>,
    color_support: ColorSupport,
) -> Option<SuggestionColor> {
    let c_str = CString::new(suggestion_str).unwrap();
    let inner =
        unsafe { color::parse_suggestion_color_zsh_autosuggest(c_str.as_ptr(), color_support.into()) };
    match inner.is_null() {
        true => None,
        false => Some(SuggestionColor {
            inner: unsafe { *inner },
        }),
    }
}
