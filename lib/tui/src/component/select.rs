use std::fmt::Display;

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use termwiz::cell::unicode_column_width;
use termwiz::color::ColorAttribute;
use termwiz::surface::{
    Change,
    CursorVisibility,
    Surface,
};

use super::shared::TextState;
use super::ComponentData;
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
    OptionSelected { id: String, option: String },
}

#[derive(Debug)]
pub struct Select {
    text_state: TextState,
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
            text_state: TextState::new(""),
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

    pub fn set_options(&mut self, options: Vec<String>) {
        self.options = options;
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner.id = id.into();
        self
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.inner.classes.push(class.into());
        self
    }

    pub fn with_text(mut self, text: impl Display) -> Self {
        self.text_state = TextState::new(text.to_string());
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

    fn update_options(&mut self) {
        self.index = None;
        self.index_offset = 0;

        let matcher = SkimMatcherV2::default();
        let mut scores = vec![];

        for i in 0..self.options.len() {
            if let Some(score) = matcher.fuzzy_match(&self.options[i], self.text_state.text()) {
                scores.push((i, score));
            }
        }

        scores.sort_by(|a, b| b.1.cmp(&a.1));

        self.sorted_options.clear();
        for (i, _) in scores {
            self.sorted_options.push(i);
        }
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

        match self.text_state.text().is_empty() {
            true => {
                let mut attributes = style.attributes();
                attributes.set_foreground(ColorAttribute::PaletteIndex(8));

                if let Some(hint) = &self.hint {
                    surface.draw_text(hint, x + 2.0, y, width - 2.0, attributes);
                }
            },
            false => {
                surface.draw_text(
                    &self.text_state.text()[self
                        .text_state
                        .grapheme_index()
                        .saturating_sub((width.round() - 3.0) as usize)..],
                    x + 2.0,
                    y,
                    width - 2.0,
                    style.attributes(),
                );
            },
        }

        if self.inner.focus {
            state.cursor_position = (
                x + 2.0 + (self.text_state.grapheme_index() as f64).min(width.round() - 3.0),
                y,
            );
            state.cursor_color = style.caret_color();
            surface.add_change(Change::CursorVisibility(CursorVisibility::Visible));
        }

        for (i, option) in self.sorted_options
            [self.index_offset..self.sorted_options.len().min(self.index_offset + self.max_rows)]
            .iter()
            .enumerate()
        {
            if i + 1 >= height as usize {
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

            let width = width - 2.0;
            let text_width = unicode_column_width(&self.options[*option], None) as f64;
            match width < text_width {
                true => surface.draw_text(
                    format!(
                        "{}...",
                        &self.options[*option].as_str()[..(width - 3.0).max(0.0) as usize].trim_end()
                    ),
                    x + 2.0,
                    y + i as f64 + 1.0,
                    width,
                    attributes,
                ),
                false => surface.draw_text(
                    self.options[*option].as_str(),
                    x + 2.0,
                    y + i as f64 + 1.0,
                    text_width,
                    attributes,
                ),
            }
        }
    }

    fn on_input_action(&mut self, _: &mut State, input_action: &InputAction) {
        match input_action {
            InputAction::Remove => {
                self.text_state.backspace();
                self.update_options();
            },
            InputAction::Submit => {
                if let Some(index) = self.index {
                    self.text_state = TextState::new(self.options[self.sorted_options[index]].clone());
                }
            },
            InputAction::Left => self.text_state.left(),
            InputAction::Right => self.text_state.right(),
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
            InputAction::Delete => {
                self.text_state.delete();
                self.update_options();
            },
            InputAction::Insert(character) => {
                self.text_state.character(*character);
                self.update_options();
            },
            InputAction::Paste(clipboard) => self.text_state.paste(clipboard),
            _ => (),
        }
    }

    fn on_focus(&mut self, state: &mut State, focus: bool) {
        self.inner.focus = focus;

        match focus {
            true => self.update_options(),
            false => {
                if !self.allow_any_text && !self.options.contains(&self.text_state.text().to_owned()) {
                    self.text_state = TextState::new("");
                }

                self.index = None;
                self.index_offset = 0;
                self.sorted_options.clear();

                if !self.text_state.text().is_empty() {
                    state.event_buffer.push(Event::Select(SelectEvent::OptionSelected {
                        id: self.inner.id.to_owned(),
                        option: self.text_state.text().to_owned(),
                    }));
                }
            },
        }
    }

    fn on_mouse_action(&mut self, _: &mut State, mouse_action: &MouseAction, x: f64, y: f64, _: f64, _: f64) {
        if self.inner.focus {
            let index = mouse_action.y - y + self.index_offset as f64;
            if index == 0.0 {
                self.text_state.on_mouse_action(mouse_action, x + 2.0);
            } else if index > 0.0 {
                self.index = Some(index as usize - 1);
                if mouse_action.just_pressed {
                    self.text_state = TextState::new(self.options[self.sorted_options[index as usize - 1]].clone());

                    self.index = None;
                    self.index_offset = 0;
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
        let mut w = unicode_column_width(self.text_state.text(), None)
            .max(
                self.options
                    .iter()
                    .fold(0, |acc, option| acc.max(unicode_column_width(option, None))),
            )
            .max(60);

        if let Some(hint) = &self.hint {
            w = w.max(unicode_column_width(hint, None));
        }

        let height = match self.inner.focus {
            true => 1.0 + self.sorted_options.len().min(self.max_rows) as f64,
            false => 1.0,
        };

        (w as f64, height)
    }

    fn as_dyn_mut(&mut self) -> &mut dyn Component {
        self
    }
}
