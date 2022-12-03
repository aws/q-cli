use std::fmt::Display;

use termwiz::color::ColorAttribute;
use termwiz::surface::Surface;
use unicode_width::UnicodeWidthStr;

use super::ComponentData;
use crate::event_loop::{
    Event,
    State,
};
use crate::input::InputAction;
use crate::surface_ext::SurfaceExt;
use crate::Component;

const MAX_ROWS: i32 = 6;

#[derive(Debug)]
pub enum SelectEvent {
    /// The user has selected an option
    OptionSelected { id: String, option: String },
}

#[derive(Debug)]
pub struct Select {
    text: String,
    hint: Option<String>,
    cursor: usize,
    cursor_offset: usize,
    index: Option<usize>,
    index_offset: usize,
    options: Vec<String>,
    sorted_options: Vec<usize>,
    validate: bool,
    inner: ComponentData,
}

impl Select {
    pub fn new(id: impl ToString, options: Vec<String>, validate: bool) -> Self {
        Self {
            text: Default::default(),
            hint: None,
            cursor: 0,
            cursor_offset: 0,
            index: Default::default(),
            index_offset: 0,
            options,
            sorted_options: vec![],
            validate,
            inner: ComponentData::new(id.to_string(), true),
        }
    }

    pub fn with_text(mut self, text: impl Display) -> Self {
        self.text = text.to_string();
        self.cursor = self.text.width();
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl Component for Select {
    fn initialize(&mut self, _: &mut State) {
        let mut w = self
            .text
            .width()
            .max(self.options.iter().fold(0, |acc, option| acc.max(option.width())))
            .max(60);

        if let Some(hint) = &self.hint {
            w = w.max(hint.width());
        }

        self.inner.width = w as f64;
        self.inner.height = 1.0;
    }

    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64, _: f64, _: f64) {
        if height <= 0.0 || width <= 0.0 {
            return;
        }

        let style = self.style(state);

        let arrow = match self.inner.focus {
            true => '▿',
            false => '▹',
        };

        surface.draw_text(arrow, x, y, 1.0, style.attributes());

        match self.text.is_empty() {
            true => {
                let mut attributes = style.attributes();
                attributes.set_foreground(ColorAttribute::PaletteIndex(8));

                if let Some(hint) = &self.hint {
                    surface.draw_text(
                        &hint.as_str()[self.cursor_offset..],
                        x + 2.0,
                        y,
                        width - 2.0,
                        attributes,
                    );
                }
            },
            false => {
                surface.draw_text(
                    &self.text.as_str()[self.cursor_offset..],
                    x + 2.0,
                    y,
                    width - 2.0,
                    style.attributes(),
                );
            },
        }

        if self.inner.focus {
            state.cursor_position = (x + 2.0 + self.cursor as f64 - self.cursor_offset as f64, y);
            state.cursor_visibility = true;
        }

        for (i, option) in self.sorted_options[self.index_offset
            ..self
                .sorted_options
                .len()
                .min(self.index_offset + usize::try_from(MAX_ROWS).unwrap())]
            .iter()
            .enumerate()
        {
            if i + 1 > height as usize {
                return;
            }

            let mut attributes = style.attributes();
            if let Some(index) = self.index {
                if i == index - self.index_offset.min(index) {
                    attributes
                        .set_foreground(style.background_color())
                        .set_background(style.caret_color());
                }
            }

            surface.draw_text(
                self.options[*option].as_str(),
                x + 2.0,
                y + i as f64 + 1.0,
                width - 2.0,
                attributes,
            );
        }
    }

    fn on_input_action(&mut self, _: &mut State, input_action: InputAction) -> bool {
        match input_action {
            InputAction::Left => self.cursor -= 1.min(self.cursor),
            InputAction::Right => self.cursor += 1.min(self.text.width() - self.cursor),
            InputAction::Up => {
                if !self.sorted_options.is_empty() {
                    match self.index {
                        Some(ref mut index) => {
                            if *index == 0 {
                                self.index_offset = self.sorted_options.len()
                                    - usize::try_from(MAX_ROWS).unwrap().min(self.sorted_options.len());
                            } else if *index == self.index_offset {
                                self.index_offset -= 1;
                            }

                            *index = (*index + self.sorted_options.len() - 1) % self.sorted_options.len();
                        },
                        None => {
                            self.index = Some(self.sorted_options.len() - 1);
                            self.index_offset = self.sorted_options.len()
                                - usize::try_from(MAX_ROWS).unwrap().min(self.sorted_options.len());
                        },
                    }
                }
            },
            InputAction::Down => {
                if !self.sorted_options.is_empty() {
                    match self.index {
                        Some(ref mut index) => {
                            if *index == self.sorted_options.len() - 1 {
                                self.index_offset = 0;
                            } else if *index == self.index_offset + usize::try_from(MAX_ROWS - 1).unwrap() {
                                self.index_offset += 1;
                            }
                            *index = (*index + 1) % self.sorted_options.len();
                        },
                        None => self.index = Some(0),
                    }
                }
            },
            InputAction::Insert(c, _) => {
                self.text.insert(self.cursor, c);
                self.cursor += 1;
                self.index = None;
                self.index_offset = 0;

                self.sorted_options
                    .retain(|option| self.options[*option].contains(&self.text));
            },
            InputAction::Remove => {
                match self.cursor == self.text.width() {
                    true => {
                        self.text.pop();
                        self.cursor -= 1.min(self.cursor);
                    },
                    false => {
                        if self.cursor == 0 {
                            return true;
                        }

                        self.text.remove(self.cursor - 1);
                        self.cursor -= 1.min(self.cursor);
                    },
                };

                self.index = None;
                self.index_offset = 0;

                self.sorted_options.clear();
                for i in 0..self.options.len() {
                    if self.options[i].contains(&self.text) {
                        self.sorted_options.push(i);
                    }
                }
            },
            InputAction::Delete => {
                match self.text.len() {
                    len if len == self.cursor + 1 => {
                        self.text.pop();
                    },
                    len if len > self.cursor + 1 => {
                        self.text.remove(self.cursor);
                    },
                    _ => (),
                }

                self.index = None;
                self.index_offset = 0;

                self.sorted_options.clear();
                for i in 0..self.options.len() {
                    if self.options[i].contains(&self.text) {
                        self.sorted_options.push(i);
                    }
                }
            },
            _ => (),
        }

        self.inner.height = 1.0 + MAX_ROWS.min(i32::try_from(self.sorted_options.len()).unwrap()) as f64;

        true
    }

    fn on_focus(&mut self, state: &mut State, focus: bool) {
        self.inner.focus = focus;

        match focus {
            true => {
                self.sorted_options = (0..self.options.len()).into_iter().collect();
                self.inner.height = 1.0 + MAX_ROWS.min(i32::try_from(self.sorted_options.len()).unwrap()) as f64;
            },
            false => {
                state.cursor_visibility = false;

                if let Some(index) = self.index {
                    self.text = self.options[self.sorted_options[index]].to_string();
                }

                if self.validate && !self.options.contains(&self.text) {
                    self.text = String::new();
                }

                self.cursor = self.text.len();
                self.index = None;
                self.index_offset = 0;

                self.sorted_options.clear();
                self.inner.height = 1.0;

                if !self.text.is_empty() {
                    state.event_buffer.push(Event::Select(SelectEvent::OptionSelected {
                        id: self.inner.id.to_owned(),
                        option: self.text.clone(),
                    }));
                }
            },
        }
    }

    fn class(&self) -> &'static str {
        "select"
    }

    fn inner(&self) -> &super::ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut super::ComponentData {
        &mut self.inner
    }
}
