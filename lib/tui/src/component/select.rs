use std::fmt::Display;

use newton::{
    Color,
    DisplayState,
};

use crate::input::InputAction;
use crate::Style;

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
    focused: bool,
    signal: Box<dyn Fn(String)>,
}

const MAX_ROWS: i32 = 6;

impl Select {
    pub fn new(options: Vec<String>, validate: bool, signal: impl Fn(String) + 'static) -> Self {
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
            focused: false,
            signal: Box::new(signal),
        }
    }

    pub fn with_text(mut self, text: impl Display) -> Self {
        self.text = text.to_string();
        self.cursor = self.text.len();
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub(crate) fn initialize(&mut self, width: &mut i32, height: &mut i32) {
        let mut w = self
            .text
            .len()
            .max(self.options.iter().fold(0, |acc, option| acc.max(option.len())))
            .max(60);
        if let Some(hint) = &self.hint {
            w = w.max(hint.len());
        }

        *width = i32::try_from(w).unwrap();
        *height = 1;
    }

    pub(crate) fn draw(&self, renderer: &mut DisplayState, style: &Style, x: i32, y: i32, width: i32, height: i32) {
        if height <= 0 || width <= 0 {
            return;
        }

        let (width, height) = match (usize::try_from(width), usize::try_from(height)) {
            (Ok(width), Ok(height)) => (width, height),
            _ => return,
        };

        let arrow = match self.focused {
            true => '▿',
            false => '▹',
        };

        renderer.draw_symbol(arrow, x, y, style.color(), style.background_color(), false);

        match self.text.is_empty() {
            true => {
                if let Some(hint) = &self.hint {
                    renderer.draw_string(
                        &hint.as_str()[self.cursor_offset..hint.len().min(width - 2 + self.cursor_offset)],
                        x + 2,
                        y,
                        Color::DarkGrey,
                        style.background_color(),
                        false,
                    );
                }
            },
            false => {
                renderer.draw_string(
                    &self.text.as_str()[self.cursor_offset..self.text.len().min(width - 2 + self.cursor_offset)],
                    x + 2,
                    y,
                    style.color(),
                    style.background_color(),
                    false,
                );

                if self.focused {
                    renderer.draw_symbol(
                        self.text.chars().nth(self.cursor).unwrap_or(' '),
                        x + 2 + i32::try_from(self.cursor).unwrap() - i32::try_from(self.cursor_offset).unwrap(),
                        y,
                        style.background_color(),
                        style.color(),
                        false,
                    );
                }
            },
        }

        for (i, option) in self.sorted_options[self.index_offset
            ..self
                .sorted_options
                .len()
                .min(self.index_offset + usize::try_from(MAX_ROWS).unwrap())]
            .iter()
            .enumerate()
        {
            if i + 1 > height {
                return;
            }

            let mut color = style.color();
            let mut background_color = style.background_color();
            if let Some(index) = self.index {
                if i == index - self.index_offset.min(index) {
                    background_color = color;
                    color = Color::Black;
                }
            }

            let option = self.options[*option].as_str();
            renderer.draw_string(
                &option[0..option.len().min(width)],
                x + 2,
                y + i32::try_from(i).unwrap() + 1,
                color,
                background_color,
                false,
            );
        }
    }

    pub(crate) fn on_input_action(&mut self, height: &mut i32, input: InputAction) {
        match input {
            InputAction::Left => self.cursor -= 1.min(self.cursor),
            InputAction::Right => self.cursor += 1.min(self.text.len() - self.cursor),
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
                match self.cursor == self.text.len() {
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

        *height = 1 + MAX_ROWS.min(i32::try_from(self.sorted_options.len()).unwrap());
    }

    pub(crate) fn on_focus(&mut self, height: &mut i32, focused: bool) {
        match focused {
            true => {
                for i in 0..self.options.len() {
                    if self.options[i].contains(&self.text) {
                        self.sorted_options.push(i);
                    }
                }
                *height = 1 + MAX_ROWS.min(i32::try_from(self.sorted_options.len()).unwrap());
            },
            false => {
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
                *height = 1;

                if !self.text.is_empty() {
                    (self.signal)(self.text.clone());
                }
            },
        }
        self.focused = focused;
    }

    pub(crate) fn on_resize(&mut self, width: i32) {
        if let Ok(width) = usize::try_from(width) {
            if self.cursor >= width {
                self.cursor_offset = self.cursor - width;
            }
        }
    }
}
