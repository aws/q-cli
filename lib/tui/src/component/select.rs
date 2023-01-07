use std::fmt::Display;

use termwiz::color::ColorAttribute;
use termwiz::surface::{
    Change,
    CursorVisibility,
    Surface,
};
use unicode_width::UnicodeWidthStr;

use super::ComponentData;
use crate::component::text_state::TextState;
use crate::event_loop::State;
use crate::input::{
    InputAction,
    MouseAction,
};
use crate::surface_ext::SurfaceExt;
use crate::{
    Component,
    Event,
};

#[derive(Debug)]
pub enum SelectEvent {
    /// The user has selected an option
    OptionSelected { id: Option<String>, option: String },
}

#[derive(Debug)]
pub struct Select {
    text: TextState,
    hint: Option<String>,
    index: Option<usize>,
    index_offset: usize,
    options: Vec<String>,
    sorted_options: Vec<usize>,
    allow_any_text: bool,
    max_rows: usize,
    inner: ComponentData,
}

impl Select {
    pub fn new(options: Vec<String>, allow_any_text: bool) -> Self {
        Self {
            text: TextState::new(""),
            hint: None,
            index: Default::default(),
            index_offset: 0,
            options,
            sorted_options: vec![],
            allow_any_text,
            max_rows: 6,
            inner: ComponentData::new("select".to_owned(), true),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner.id = Some(id.into());
        self
    }

    pub fn with_text(mut self, text: impl Display) -> Self {
        self.text = TextState::new(text.to_string());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn with_max_rows(mut self, max_rows: usize) -> Self {
        self.max_rows = max_rows.max(1);
        self
    }
}

impl Component for Select {
    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64) {
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
                    surface.draw_text(hint, x + 2.0, y, width - 2.0, attributes);
                }
            },
            false => {
                surface.draw_text(
                    &self.text.as_str()[self.text.cursor.saturating_sub((width.round() - 3.0) as usize)..],
                    x + 2.0,
                    y,
                    width - 2.0,
                    style.attributes(),
                );
            },
        }

        if self.inner.focus {
            state.cursor_position = (x + 2.0 + (self.text.cursor as f64).min(width.round() - 3.0), y);
            state.cursor_color = style.caret_color();
            surface.add_change(Change::CursorVisibility(CursorVisibility::Visible));
        }

        for (i, option) in self.sorted_options
            [self.index_offset..self.sorted_options.len().min(self.index_offset + self.max_rows)]
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
                        .set_background(style.color())
                        .set_foreground(ColorAttribute::PaletteIndex(0));
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

    fn on_input_action(&mut self, _: &mut State, input_action: &InputAction) {
        if self.text.on_input_action(input_action).is_err() {
            return;
        }

        match input_action {
            InputAction::Submit => {
                if let Some(index) = self.index {
                    self.text = TextState::new(self.options[self.sorted_options[index]].clone());
                }
            },
            InputAction::Up => {
                if !self.sorted_options.is_empty() {
                    match self.index {
                        Some(ref mut index) => {
                            if *index == 0 {
                                self.index_offset =
                                    self.sorted_options.len() - self.max_rows.min(self.sorted_options.len());
                            } else if *index == self.index_offset {
                                self.index_offset -= 1;
                            }

                            *index = (*index + self.sorted_options.len() - 1) % self.sorted_options.len();
                        },
                        None => {
                            self.index = Some(self.sorted_options.len() - 1);
                            self.index_offset =
                                self.sorted_options.len() - self.max_rows.min(self.sorted_options.len());
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
                            } else if *index == self.index_offset + (self.max_rows - 1) {
                                self.index_offset += 1;
                            }
                            *index = (*index + 1) % self.sorted_options.len();
                        },
                        None => self.index = Some(0),
                    }
                }
            },
            InputAction::Insert(_) => {
                self.index = None;
                self.index_offset = 0;
                self.sorted_options
                    .retain(|option| self.options[*option].contains(&*self.text));
            },
            InputAction::Remove => {
                self.index = None;
                self.index_offset = 0;

                self.sorted_options.clear();
                for i in 0..self.options.len() {
                    if self.options[i].contains(&*self.text) {
                        self.sorted_options.push(i);
                    }
                }
            },
            InputAction::Delete => {
                self.index = None;
                self.index_offset = 0;

                self.sorted_options.clear();
                for i in 0..self.options.len() {
                    if self.options[i].contains(&*self.text) {
                        self.sorted_options.push(i);
                    }
                }
            },
            _ => (),
        }
    }

    fn on_focus(&mut self, state: &mut State, focus: bool) {
        self.inner.focus = focus;

        match focus {
            true => {
                self.sorted_options.clear();
                for i in 0..self.options.len() {
                    if self.options[i].contains(&*self.text) {
                        self.sorted_options.push(i);
                    }
                }
            },
            false => {
                if !self.allow_any_text && !self.options.contains(&self.text) {
                    self.text = TextState::new("");
                }

                self.text.cursor = self.text.len();
                self.index = None;
                self.index_offset = 0;

                self.sorted_options.clear();

                if !self.text.is_empty() {
                    state.event_buffer.push(Event::Select(SelectEvent::OptionSelected {
                        id: self.inner.id.to_owned(),
                        option: self.text.clone(),
                    }));
                }
            },
        }
    }

    fn on_mouse_action(&mut self, _: &mut State, mouse_action: &MouseAction, x: f64, y: f64, _: f64, _: f64) {
        if self.inner.focus {
            let index = mouse_action.y - y + self.index_offset as f64;
            if index == 0.0 {
                self.text.on_mouse_action(mouse_action, x + 2.0);
            } else if index > 0.0 {
                self.index = Some(index as usize - 1);
                if mouse_action.just_pressed {
                    self.text = TextState::new(self.options[self.sorted_options[index as usize - 1]].clone());
                    self.sorted_options.clear();
                }
            }
        }
    }

    fn inner(&self) -> &super::ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut super::ComponentData {
        &mut self.inner
    }

    fn size(&self, _: &mut State) -> (f64, f64) {
        let mut w = self
            .text
            .width()
            .max(self.options.iter().fold(0, |acc, option| acc.max(option.width())))
            .max(60);

        if let Some(hint) = &self.hint {
            w = w.max(hint.width());
        }

        let height = match self.inner.focus {
            true => 1.0 + self.sorted_options.len().min(self.max_rows) as f64,
            false => 1.0,
        };

        (w as f64, height)
    }
}
