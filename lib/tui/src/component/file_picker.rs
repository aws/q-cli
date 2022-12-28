use std::path::{
    Path,
    PathBuf,
};

use termwiz::color::ColorAttribute;
use termwiz::input::MouseButtons;
use termwiz::surface::{
    Change,
    CursorVisibility,
    Surface,
};

use super::text_state::TextState;
use super::ComponentData;
use crate::event_loop::{
    Event,
    State,
};
use crate::input::{
    InputAction,
    MouseAction,
};
use crate::surface_ext::SurfaceExt;
use crate::Component;

const MAX_ROWS: i32 = 8;

#[derive(Debug)]
pub enum FilePickerEvent {
    /// The user has either typed a valid or invalid path or selected a valid one
    FilePathChanged { id: String, path: PathBuf },
}

#[derive(Debug)]
pub struct FilePicker {
    text: TextState,
    _files: bool,
    _folders: bool,
    _extensions: Vec<String>,
    options: Vec<String>,
    preview: Vec<String>,
    index: Option<usize>,
    index_offset: usize,
    inner: ComponentData,
}

impl FilePicker {
    pub fn new(id: impl ToString, files: bool, folders: bool, extensions: Vec<String>) -> Self {
        Self {
            text: TextState::default(),
            _files: files,
            _folders: folders,
            _extensions: extensions,
            options: vec![],
            preview: vec![],
            index: None,
            index_offset: 0,
            inner: ComponentData::new("select".to_owned(), id.to_string(), true),
        }
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        *self.text = path.into();
        self.text.cursor = self.text.len();
        self
    }

    fn update_options(&mut self) {
        let path = Path::new(self.text.as_str());
        if path.exists() {
            self.options.clear();

            if let Ok(dir) = std::fs::read_dir(path) {
                for file in dir.flatten() {
                    if let Some(file_name) = file.file_name().to_str() {
                        self.options.push(file_name.to_owned())
                    }
                }

                self.options.sort_by(|a, b| {
                    let apath = path.join(a);
                    let bpath = path.join(b);

                    match (apath.is_dir(), bpath.is_dir()) {
                        (true, true) => a.cmp(b),
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        (false, false) => a.cmp(b),
                    }
                })
            }
        }
    }

    fn update_preview(&mut self) {
        self.preview.clear();

        let index = self.index.unwrap_or(0);
        let path = Path::new(self.text.as_str());
        if let Some(option) = self.options.get(index) {
            let path = path.join(option);
            if let Ok(dir) = std::fs::read_dir(&path) {
                for file in dir.flatten() {
                    if let Some(file_name) = file.file_name().to_str() {
                        self.preview.push(file_name.to_owned())
                    }
                }
            }

            self.preview.sort_by(|a, b| {
                let apath = path.join(a);
                let bpath = path.join(b);

                match (apath.is_dir(), bpath.is_dir()) {
                    (true, true) => a.cmp(b),
                    (true, false) => std::cmp::Ordering::Less,
                    (false, true) => std::cmp::Ordering::Greater,
                    (false, false) => a.cmp(b),
                }
            })
        }
    }
}

impl Component for FilePicker {
    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64, _: f64, _: f64) {
        if height <= 0.0 || width <= 0.0 {
            return;
        }

        let style = self.style(state);

        surface.draw_text(self.text.as_str(), x, y, width, style.attributes());

        if height as usize > 1 {
            surface.draw_rect('─', x, y + 1.0, width, 1.0, style.attributes());
            surface.draw_text('┬', x + width * 0.5 - 1.0, y + 1.0, 1.0, style.attributes());
        }
        surface.draw_rect(
            '│',
            x + width * 0.5 - 1.0,
            y + 2.0,
            1.0,
            height - 2.0,
            style.attributes(),
        );

        for (i, option) in self.options[self.index_offset
            ..self
                .options
                .len()
                .min(self.index_offset + usize::try_from(MAX_ROWS).unwrap())]
            .iter()
            .enumerate()
        {
            if i + 3 > height as usize {
                break;
            }

            let mut attributes = style.attributes();
            if !Path::new(self.text.as_str()).join(option).is_dir() {
                attributes
                    .set_foreground(ColorAttribute::PaletteIndex(8))
                    .set_background(ColorAttribute::Default);
            };

            if let Some(index) = self.index {
                if i == index - self.index_offset.min(index) {
                    attributes
                        .set_background(attributes.foreground())
                        .set_foreground(ColorAttribute::PaletteIndex(0));
                }
            }

            surface.draw_text(option, x + 1.0, y + i as f64 + 2.0, width * 0.5 - 3.0, attributes);
        }

        if let Some(index) = self.index {
            if let Some(option) = self.options.get(index) {
                let path = Path::new(self.text.as_str()).join(option);
                for (i, preview) in self.preview.iter().enumerate() {
                    if i + 3 > height as usize {
                        break;
                    }

                    let mut attributes = style.attributes();
                    if !path.join(preview).is_dir() {
                        attributes
                            .set_foreground(ColorAttribute::PaletteIndex(8))
                            .set_background(ColorAttribute::Default);
                    };

                    surface.draw_text(
                        preview,
                        x + 1.0 + width * 0.5,
                        y + i as f64 + 2.0,
                        width * 0.5 - 3.0,
                        style.attributes(),
                    );
                }
            }
        }

        if self.index.is_none() && self.inner.focus {
            state.cursor_position = (x + self.text.cursor as f64, y);
            state.cursor_color = style.caret_color();
            surface.add_change(Change::CursorVisibility(CursorVisibility::Visible));
        }
    }

    fn on_input_action(&mut self, state: &mut State, input_action: &InputAction) {
        if self.index.is_none() {
            self.text.on_input_action(input_action).ok();
            self.update_options();
            self.update_preview();
        }

        match input_action {
            InputAction::Up => {
                if !self.options.is_empty() {
                    match self.index {
                        Some(index) => {
                            if index == 0 {
                                self.index = None;
                            } else if index == self.index_offset {
                                self.index_offset -= 1;
                                self.index = Some((index + self.options.len() - 1) % self.options.len());
                            } else {
                                self.index = Some((index + self.options.len() - 1) % self.options.len());
                            }
                        },
                        None => {
                            self.index = Some(self.options.len() - 1);
                            self.index_offset =
                                self.options.len() - usize::try_from(MAX_ROWS - 1).unwrap().min(self.options.len());
                        },
                    }

                    self.update_preview();
                }
            },
            InputAction::Down => {
                if !self.options.is_empty() {
                    match self.index {
                        Some(index) => {
                            if index == self.options.len() - 1 {
                                self.index = Some(0);
                                self.index_offset = 0;
                            } else if index == self.index_offset + usize::try_from(MAX_ROWS - 2).unwrap() {
                                self.index = Some((index + 1) % self.options.len());
                                self.index_offset += 1;
                            } else {
                                self.index = Some((index + 1) % self.options.len());
                            }
                        },
                        None => {
                            self.index = Some(0);
                            self.index_offset = 0;
                        },
                    }

                    self.update_preview();
                }
            },
            InputAction::Submit | InputAction::Right => {
                if !self.options.is_empty() {
                    if let Some(index) = self.index {
                        let path = Path::new(self.text.as_str()).join(&self.options[index]);
                        if let Some(path_str) = path.to_str() {
                            *self.text = path_str.to_owned();
                            self.text.cursor = self.text.len();

                            state
                                .event_buffer
                                .push(Event::FilePicker(FilePickerEvent::FilePathChanged {
                                    id: self.inner.id.to_owned(),
                                    path: path.to_owned(),
                                }));

                            self.index = Some(0);
                            self.index_offset = 0;

                            self.update_options();
                            self.update_preview();
                        };
                    }
                }
            },
            InputAction::Left => {
                if self.index.is_some() {
                    if let Some(path) = PathBuf::new().join(self.text.as_str()).parent() {
                        if let Some(path_str) = path.to_str() {
                            *self.text = path_str.to_owned();
                            self.text.cursor = self.text.len();

                            state
                                .event_buffer
                                .push(Event::FilePicker(FilePickerEvent::FilePathChanged {
                                    id: self.inner.id.to_owned(),
                                    path: path.to_owned(),
                                }));

                            self.index = Some(0);
                            self.index_offset = 0;

                            self.update_options();
                            self.update_preview();
                        }
                    }
                }
            },
            _ => (),
        }
    }

    fn on_mouse_action(&mut self, state: &mut State, mouse_action: &MouseAction, x: f64, y: f64, _: f64, _: f64) {
        if self.inner.focus {
            let index = mouse_action.y - y + self.index_offset as f64;
            if index == 0.0 && mouse_action.buttons.contains(MouseButtons::LEFT) {
                self.index = None;
                self.text.on_mouse_action(mouse_action, x);
            } else if index > 1.0 && (index as usize - 2) < self.options.len() {
                match mouse_action.just_pressed {
                    true => {
                        let path = Path::new(self.text.as_str()).join(&self.options[index as usize - 2]);
                        if let Some(path_str) = path.to_str() {
                            *self.text = path_str.to_owned();
                            self.text.cursor = self.text.len();

                            state
                                .event_buffer
                                .push(Event::FilePicker(FilePickerEvent::FilePathChanged {
                                    id: self.inner.id.to_owned(),
                                    path: path.to_owned(),
                                }));

                            self.index = None;
                            self.index_offset = 0;

                            self.update_options();
                            self.update_preview();
                        }
                    },
                    false => {
                        self.index = Some(index as usize - 2);
                        self.update_preview();
                    },
                }
            }
        }
    }

    fn on_focus(&mut self, _: &mut State, focus: bool) {
        self.inner.focus = focus;

        match focus {
            true => {
                self.index = Some(0);
                self.index_offset = 0;

                self.update_options();
                self.update_preview();
            },
            false => {
                self.options.clear();
                self.preview.clear();
            },
        }
    }

    fn inner(&self) -> &super::ComponentData {
        &self.inner
    }

    fn inner_mut(&mut self) -> &mut super::ComponentData {
        &mut self.inner
    }

    fn size(&self, _: &mut State) -> (f64, f64) {
        let height = match self.inner.focus {
            true => 2.0 + MAX_ROWS as f64,
            false => 1.0,
        };

        (120.0, height)
    }
}
