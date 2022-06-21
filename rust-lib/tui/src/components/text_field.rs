use newton::{
    Color,
    ControlFlow,
    DisplayState,
    KeyCode,
};

use crate::{
    BorderStyle,
    Component,
    Event,
    Style,
    StyleContext,
    StyleSheet,
};

#[derive(Debug, Default)]
pub struct TextField {
    pub text: String,
    cursor: usize,
    offset: usize,
    pub hint: Option<String>,
    obfuscated: bool,
    style: Style,
}

impl TextField {
    pub const STYLE_CLASS: &'static str = "textfield";

    stylable!();

    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            offset: 0,
            hint: None,
            obfuscated: false,
            style: Default::default(),
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.cursor = self.text.len();
        self
    }

    pub fn obfuscated(mut self, obfuscated: bool) -> Self {
        self.obfuscated = obfuscated;
        self
    }
}

impl Component for TextField {
    fn update(
        &mut self,
        renderer: &mut DisplayState,
        style_sheet: &StyleSheet,
        _: &mut ControlFlow,
        focused: bool,
        event: Event,
    ) {
        let context = StyleContext { focused, hover: false };

        match event {
            Event::Initialize => {
                let style = self.style(style_sheet, context);

                self.style = style
                    .with_width(32)
                    .with_height(1)
                    .with_max_width(128)
                    .with_max_height(u16::try_from(16).unwrap());
            },
            Event::Draw {
                mut x,
                mut y,
                mut width,
                mut height,
            } => {
                let style = self.style(style_sheet, context);
                style.draw_container(&mut x, &mut y, &mut width, &mut height, renderer);

                if self.cursor >= width.into() {
                    self.offset = self.cursor - usize::from(width);
                }

                match self.text.is_empty() {
                    true => match &self.hint {
                        Some(hint) => renderer.draw_string(
                            &hint.as_str()[self.offset..hint.len().min(usize::from(width) + self.offset)],
                            x,
                            y,
                            Color::DarkGrey,
                            style.background_color(),
                        ),
                        None => renderer,
                    },
                    false => {
                        match self.obfuscated {
                            true => renderer.draw_string(
                                "*".repeat(self.text.len().min(width.into())),
                                x,
                                y,
                                style.color(),
                                style.background_color(),
                            ),
                            false => renderer.draw_string(
                                &self.text.as_str()[self.offset..self.text.len().min(usize::from(width) + self.offset)],
                                x,
                                y,
                                style.color(),
                                style.background_color(),
                            ),
                        };

                        if focused {
                            renderer.draw_symbol(
                                self.text.chars().nth(self.cursor).unwrap_or(' '),
                                x + u16::try_from(self.cursor).unwrap() - u16::try_from(self.offset).unwrap(),
                                y,
                                style.background_color(),
                                style.color(),
                            );
                        }

                        renderer
                    },
                };
            },
            Event::KeyPressed { code, .. } => {
                if focused {
                    match code {
                        KeyCode::Left => self.cursor -= 1.min(self.cursor),
                        KeyCode::Right => self.cursor += 1.min(self.text.len() - self.cursor),
                        KeyCode::Backspace => match self.cursor == self.text.len() {
                            true => {
                                self.text.pop();
                                self.cursor -= 1.min(self.cursor);
                            },
                            false => {
                                if self.cursor == 0 {
                                    return;
                                }

                                self.text.remove(self.cursor - 1);
                                self.cursor -= 1.min(self.cursor);
                            },
                        },
                        KeyCode::Char(c) => {
                            self.text.insert(self.cursor, c);
                            self.cursor += 1;
                        },
                        _ => (),
                    }
                }
            },
            _ => (),
        }
    }

    fn interactive(&self) -> bool {
        true
    }

    fn inline_style(&self) -> Option<Style> {
        Some(self.style)
    }

    fn desired_width(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_horizontal() + 32
    }

    fn desired_height(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_vertical() + 1
    }

    fn class(&self) -> &str {
        Self::STYLE_CLASS
    }
}
