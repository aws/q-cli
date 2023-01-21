use std::path::{
    Path,
    PathBuf,
};

use termwiz::color::ColorAttribute;
use termwiz::surface::{
    Change,
    CursorVisibility,
    Surface,
};
use unicode_width::UnicodeWidthStr;

use super::shared::TextState;
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
    FilePathChanged { id: Option<String>, path: PathBuf },
}

#[derive(Debug)]
pub struct FilePicker {
    text_state: TextState,
    hovered_component: Option<usize>,
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
    pub fn new(files: bool, folders: bool, extensions: Vec<String>) -> Self {
        Self {
            text_state: TextState::default(),
            hovered_component: None,
            _files: files,
            _folders: folders,
            _extensions: extensions,
            options: vec![],
            preview: vec![],
            index: None,
            index_offset: 0,
            inner: ComponentData::new("select".to_owned(), true),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner.id = Some(id.into());
        self
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.inner.classes.push(class.into());
        self
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.text_state.set_text(path.into());
        self
    }

    fn update_options(&mut self) {
        let path = Path::new(self.text_state.text());
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
        let path = Path::new(self.text_state.text());
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
    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64) {
        if height <= 0.0 || width <= 0.0 {
            return;
        }

        let style = self.style(state);

        let mut path_x = x;
        for (i, component) in self.text_state.text().split('/').enumerate() {
            if component.is_empty() {
                continue;
            }

            surface.draw_text('/', path_x, y, width - path_x, style.attributes());
            path_x += 1.0;

            let mut attributes = style.attributes();
            if let Some(hovered) = self.hovered_component {
                if i == hovered {
                    attributes
                        .set_background(attributes.foreground())
                        .set_foreground(ColorAttribute::PaletteIndex(0));
                }
            }

            surface.draw_text(
                &component[self
                    .text_state
                    .grapheme_index()
                    .saturating_sub((width.round() - 1.0) as usize)
                    .min(component.len())..],
                path_x,
                y,
                width - path_x,
                attributes,
            );
            path_x += component.width() as f64;
        }

        if self.text_state.text().ends_with('/') {
            surface.draw_text('/', path_x, y, width - path_x, style.attributes());
        }

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
            if !Path::new(self.text_state.text()).join(option).is_dir() {
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
                let path = Path::new(self.text_state.text()).join(option);
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
            state.cursor_position = (
                x + (self.text_state.grapheme_index() as f64).min(width.round() - 1.0),
                y,
            );
            state.cursor_color = style.caret_color();
            surface.add_change(Change::CursorVisibility(CursorVisibility::Visible));
        }
    }

    fn on_input_action(&mut self, state: &mut State, input_action: &InputAction) {
        if let InputAction::Insert(_) | InputAction::Remove = input_action {
            self.index = None;
        }

        match input_action {
            InputAction::Remove => self.text_state.backspace(),
            InputAction::Submit | InputAction::Right => {
                match self.index {
                    Some(index) => {
                        if !self.options.is_empty() {
                            let path = Path::new(self.text_state.text()).join(&self.options[index]);
                            if let Some(path_str) = path.to_str() {
                                self.text_state.set_text(path_str);
                                //
                                state
                                    .event_buffer
                                    .push(Event::FilePicker(FilePickerEvent::FilePathChanged {
                                        id: self.inner.id.to_owned(),
                                        path: path.to_owned(),
                                    }));
                                //
                                self.index = Some(0);
                                self.index_offset = 0;
                                //
                                self.update_options();
                                self.update_preview();
                            };
                        }
                    },
                    None => self.text_state.right(),
                }
            },
            InputAction::Left => match self.index {
                Some(_) => {
                    if let Some(path) = PathBuf::new().join(self.text_state.text()).parent() {
                        if let Some(path_str) = path.to_str() {
                            self.text_state.set_text(path_str);
                            //
                            state
                                .event_buffer
                                .push(Event::FilePicker(FilePickerEvent::FilePathChanged {
                                    id: self.inner.id.to_owned(),
                                    path: path.to_owned(),
                                }));
                            //
                            self.index = Some(0);
                            self.index_offset = 0;
                            //
                            self.update_options();
                            self.update_preview();
                        }
                    }
                },
                None => self.text_state.left(),
            },
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
                    //
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
                    //
                    self.update_preview();
                }
            },
            InputAction::Delete => self.text_state.delete(),
            InputAction::Insert(character) => self.text_state.character(*character),
            InputAction::Paste(clipboard) => self.text_state.paste(clipboard),
            _ => (),
        }
    }

    fn on_mouse_action(&mut self, state: &mut State, mouse_action: &MouseAction, x: f64, y: f64, _: f64, _: f64) {
        if self.inner.focus {
            self.hovered_component = None;

            let index = mouse_action.y - y + self.index_offset as f64;
            if index == 0.0 {
                let index = mouse_action.x - x;

                let mut x = 0.0;
                let mut text = String::default();
                for (i, slice) in self.text_state.text().split('/').enumerate() {
                    if !slice.is_empty() {
                        x += 1.0;
                        text.push('/');
                    }

                    if index >= x && index < x + slice.width() as f64 {
                        self.hovered_component = Some(i);

                        if mouse_action.just_pressed {
                            text.push_str(slice);
                            text.push('/');
                            self.text_state.set_text(text);

                            self.update_options();
                            self.update_preview();

                            self.index = Some(0);
                        }

                        break;
                    }

                    x += slice.width() as f64;
                    text.push_str(slice);
                }

                if index > self.text_state.text().width() as f64 && mouse_action.just_pressed {
                    self.index = None;
                }
            } else if index > 1.0 && (index as usize - 2) < self.options.len() {
                match mouse_action.just_pressed {
                    true => {
                        let path = Path::new(self.text_state.text()).join(&self.options[index as usize - 2]);
                        let dir = path.is_dir();
                        if let Some(path_str) = path.to_str() {
                            self.text_state.set_text(path_str);
                            if dir {
                                self.text_state.character('/');
                            }

                            state
                                .event_buffer
                                .push(Event::FilePicker(FilePickerEvent::FilePathChanged {
                                    id: self.inner.id.to_owned(),
                                    path: path.to_owned(),
                                }));

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
                self.index_offset = 0;
                self.options.clear();
                self.preview.clear();
                self.hovered_component = None;
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
            true => {
                let path = Path::new(self.text_state.text());
                match path.is_file() {
                    true => 1.0,
                    false => 2.0 + MAX_ROWS as f64,
                }
            },
            false => 1.0,
        };

        (120.0, height)
    }

    fn as_dyn_mut(&mut self) -> &mut dyn Component {
        self
    }
}
