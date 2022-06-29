use std::char::MAX;

use newton::{
    Color,
    ControlFlow,
    DisplayState,
};

use crate::{
    BorderStyle,
    Component,
    Event,
    KeyCode,
    Style,
    StyleContext,
    StyleSheet,
};

pub struct Select {
    pub text: String,
    pub hint: Option<String>,
    cursor: usize,
    cursor_offset: usize,
    index: Option<usize>,
    index_offset: usize,
    options: Vec<String>,
    sorted_options: Vec<usize>,
    validate: bool,
    style: Style,
}

const MAX_ROWS: u16 = 6;

impl Select {
    const STYLE_CLASS: &'static str = "select";

    stylable!();

    pub fn new(options: Vec<String>, validate: bool) -> Self {
        let sorted_options = vec![];
        // for i in 0..options.len() {
        //    sorted_options.push(i);
        //}

        Self {
            text: Default::default(),
            hint: None,
            cursor: 0,
            cursor_offset: 0,
            index: Default::default(),
            index_offset: 0,
            options,
            sorted_options,
            validate,
            style: Default::default(),
        }
    }

    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self.cursor = self.text.len();
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

impl Component for Select {
    fn update(
        &mut self,
        renderer: &mut DisplayState,
        style_sheet: &StyleSheet,
        _: &mut ControlFlow,
        focused: bool,
        event: Event,
    ) {
        let context = StyleContext { focused, hover: false };

        if focused {
            self.sorted_options.clear();
            for i in 0..self.options.len() {
                if self.options[i].contains(&self.text) {
                    self.sorted_options.push(i);
                }
            }
        }

        match event {
            Event::Initialize => {
                let style = self.style(style_sheet, context);

                self.style = style
                    .with_width(style.width.unwrap_or_else(|| {
                        self.options
                            .iter()
                            .fold(0, |acc, option| acc.max(u16::try_from(option.len()).unwrap()))
                    }))
                    .with_height(
                        style
                            .height
                            .unwrap_or_else(|| u16::try_from(self.options.len()).unwrap()),
                    )
                    .with_max_width(style.max_width.unwrap_or(128))
                    .with_max_height(
                        style
                            .max_height
                            .unwrap_or_else(|| u16::try_from(self.options.len() * 2).unwrap()),
                    );
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
                    self.cursor_offset = self.cursor - usize::from(width);
                }

                let arrow = match focused {
                    true => '▿',
                    false => '▹',
                };

                renderer.draw_symbol(arrow, x, y, style.color(), style.background_color());

                match self.text.is_empty() {
                    true => {
                        if let Some(hint) = &self.hint {
                            renderer.draw_string(
                                &hint.as_str()
                                    [self.cursor_offset..hint.len().min(usize::from(width) + self.cursor_offset)],
                                x + 2,
                                y,
                                Color::DarkGrey,
                                style.background_color(),
                            );
                        }
                    },
                    false => {
                        renderer.draw_string(
                            &self.text.as_str()
                                [self.cursor_offset..self.text.len().min(usize::from(width) + self.cursor_offset)],
                            x + 2,
                            y,
                            style.color(),
                            style.background_color(),
                        );

                        if focused {
                            renderer.draw_symbol(
                                self.text.chars().nth(self.cursor).unwrap_or(' '),
                                x + 2 + u16::try_from(self.cursor).unwrap()
                                    - u16::try_from(self.cursor_offset).unwrap(),
                                y,
                                style.background_color(),
                                style.color(),
                            );
                        }
                    },
                }

                for (i, option) in self.sorted_options
                    [self.index_offset..self.sorted_options.len().min(self.index_offset + usize::from(MAX_ROWS))]
                    .iter()
                    .enumerate()
                {
                    if i + 1 > usize::from(height) {
                        return;
                    }

                    let mut color = style.color();
                    let mut background_color = style.background_color();
                    if let Some(index) = self.index {
                        if i == index - self.index_offset.min(index) {
                            std::mem::swap(&mut color, &mut background_color);
                        }
                    }

                    let option = self.options[*option].as_str();
                    renderer.draw_string(
                        &option[0..option.len().min(usize::from(width))],
                        x + 2,
                        y + u16::try_from(i).unwrap() + 1,
                        color,
                        background_color,
                    );
                }
            },
            Event::KeyPressed { code, .. } => {
                if focused {
                    match code {
                        KeyCode::Up => {
                            if !self.sorted_options.is_empty() {
                                match self.index {
                                    Some(ref mut index) => {
                                        if *index == 0 {
                                            self.index_offset = self.sorted_options.len()
                                                - usize::from(MAX_ROWS).min(self.sorted_options.len());
                                        } else if *index == self.index_offset {
                                            self.index_offset -= 1;
                                        }

                                        *index = (*index + self.sorted_options.len() - 1) % self.sorted_options.len();
                                    },
                                    None => {
                                        self.index = Some(self.sorted_options.len() - 1);
                                        self.index_offset = self.sorted_options.len()
                                            - usize::from(MAX_ROWS).min(self.sorted_options.len());
                                    },
                                }
                            }
                        },
                        KeyCode::Down => {
                            if !self.sorted_options.is_empty() {
                                match self.index {
                                    Some(ref mut index) => {
                                        if *index == self.sorted_options.len() - 1 {
                                            self.index_offset = 0;
                                        } else if *index == self.index_offset + usize::from(MAX_ROWS - 1) {
                                            self.index_offset += 1;
                                        }
                                        *index = (*index + 1) % self.sorted_options.len();
                                    },
                                    None => self.index = Some(0),
                                }
                            }
                        },
                        KeyCode::Left => self.cursor -= 1.min(self.cursor),
                        KeyCode::Right => self.cursor += 1.min(self.text.len() - self.cursor),
                        KeyCode::Backspace | KeyCode::Delete => {
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
                            // self.sorted_options.clear();
                            // for i in 0..self.options.len() {
                            //    if self.options[i].contains(&self.text) {
                            //        self.sorted_options.push(i);
                            //    }
                            //}
                        },
                        KeyCode::Char(c) => {
                            self.text.insert(self.cursor, c);
                            self.cursor += 1;
                            self.index = None;
                            self.index_offset = 0;
                            // self.sorted_options
                            //    .retain(|option| self.options[*option].contains(&self.text));
                        },
                        KeyCode::Enter | KeyCode::Tab | KeyCode::BackTab => {
                            if let Some(index) = self.index {
                                self.text = self.options[self.sorted_options[index]].to_string();
                                self.cursor = self.text.len();
                                self.index = None;
                            }

                            if self.validate && !self.options.contains(&self.text) {
                                self.text = String::new();
                            }

                            self.sorted_options.clear();
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
        self.style(style_sheet, context).spacing_horizontal()
            + self
                .options
                .iter()
                .fold(0, |acc, option| acc.max(u16::try_from(option.len() + 2).unwrap()))
                .min(32)
                .min(u16::try_from(self.hint.as_ref().and_then(|hint| Some(hint.len())).unwrap_or(0)).unwrap())
            + 2
    }

    fn desired_height(&self, style_sheet: &StyleSheet, context: StyleContext) -> u16 {
        self.style(style_sheet, context).spacing_vertical()
            + u16::try_from(self.sorted_options.len().min(MAX_ROWS.into())).unwrap()
            + 1
    }

    fn class(&self) -> &str {
        Self::STYLE_CLASS
    }
}
