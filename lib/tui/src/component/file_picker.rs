use std::ops::Deref;
use std::path::{
    Path,
    PathBuf,
};

use termwiz::cell::unicode_column_width;
use termwiz::color::ColorAttribute;
use termwiz::surface::{
    Change,
    CursorVisibility,
    Surface,
};

use super::shared::{
    ListState,
    TextState,
};
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

#[derive(Debug)]
pub enum FilePickerEvent {
    /// The user has either typed a valid or invalid path or selected a valid one
    FilePathChanged { id: String, path: PathBuf },
}

#[derive(Debug)]
pub struct FilePicker {
    text_state: TextState,
    list_state: ListState,
    typing: bool,
    preview_state: ListState,
    folders_only: bool,
    valid_extensions: Vec<String>,
    inner: ComponentData,
}

impl FilePicker {
    pub fn new(folders_only: bool, valid_extensions: Vec<String>) -> Self {
        Self {
            text_state: TextState::default(),
            list_state: ListState::new(vec![]),
            typing: true,
            preview_state: ListState::new(vec![]),
            folders_only,
            valid_extensions,
            inner: ComponentData::new("select".to_owned(), true),
        }
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.inner.id = id.into();
        self
    }

    pub fn with_class(mut self, class: impl Into<String>) -> Self {
        self.inner.classes.push(class.into());
        self
    }

    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.text_state.set_text(path.into());
        self.text_state.character('/');
        self
    }

    fn update_options(&mut self) {
        let path = Path::new(self.text_state.text());
        if path.exists() {
            if let Ok(dir) = std::fs::read_dir(path) {
                let mut options = vec![];
                for file in dir.flatten() {
                    if let Some(file_name) = file.file_name().to_str() {
                        options.push(file_name.to_owned())
                    }
                }

                if self.folders_only {
                    options.retain(|option| path.join(option).is_dir());
                }

                if !self.valid_extensions.is_empty() {
                    options.retain(|option| {
                        path.join(option).is_dir()
                            || self.valid_extensions.contains(
                                &path
                                    .join(option)
                                    .extension()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .deref()
                                    .to_owned(),
                            )
                    })
                }

                options.sort_by(|a, b| {
                    let apath = path.join(a);
                    let bpath = path.join(b);

                    match (apath.is_dir(), bpath.is_dir()) {
                        (true, true) => a.cmp(b),
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        (false, false) => a.cmp(b),
                    }
                });

                self.list_state = ListState::new(options);
            }
        }
    }

    fn update_preview(&mut self) {
        if let Some(selection) = self.list_state.selection() {
            let path = match self.text_state.text().ends_with('/') {
                true => Path::new(self.text_state.text()),
                false => match Path::new(self.text_state.text()).parent() {
                    Some(parent) => parent,
                    None => return,
                },
            }
            .join(selection);

            if let Ok(dir) = std::fs::read_dir(&path) {
                let mut options = vec![];
                for file in dir.flatten() {
                    if let Some(file_name) = file.file_name().to_str() {
                        options.push(file_name.to_owned())
                    }
                }

                if self.folders_only {
                    options.retain(|option| path.join(option).is_dir());
                }

                if !self.valid_extensions.is_empty() {
                    options.retain(|option| {
                        path.join(option).is_dir()
                            || self.valid_extensions.contains(
                                &path
                                    .join(option)
                                    .extension()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .deref()
                                    .to_owned(),
                            )
                    })
                }

                options.sort_by(|a, b| {
                    let apath = path.join(a);
                    let bpath = path.join(b);

                    match (apath.is_dir(), bpath.is_dir()) {
                        (true, true) => a.cmp(b),
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        (false, false) => a.cmp(b),
                    }
                });

                self.preview_state = ListState::new(options);
            } else {
                self.preview_state.clear();
            }
        }
    }

    fn sort_options(&mut self) {
        let path = Path::new(self.text_state.text());
        if self.text_state.text().ends_with('/') || self.text_state.text().ends_with('\\') {
            self.update_options();
            self.update_preview();
        } else if let Some(file_name) = path.file_name() {
            if let Some(file_name) = file_name.to_str() {
                self.list_state.sort("");
                self.list_state.sort(file_name);
                self.update_preview();
            }
        }
    }
}

impl Component for FilePicker {
    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64) {
        if height <= 0.0 || width <= 0.0 {
            return;
        }

        let style = self.style(state);

        let text_width = unicode_column_width(self.text_state.text(), None);
        surface.draw_text(
            &self.text_state.text()[text_width.saturating_sub(width as usize)..],
            x,
            y,
            width,
            style.attributes(),
        );

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

        for (i, option) in self.list_state.sorted_options().iter().enumerate() {
            if i.saturating_add(3) > height as usize {
                break;
            }

            let path = match self.text_state.text().ends_with('/') {
                true => Path::new(self.text_state.text()),
                false => match Path::new(self.text_state.text()).parent() {
                    Some(parent) => parent,
                    None => return,
                },
            }
            .join(option);

            let mut attributes = style.attributes();
            if !path.is_dir() {
                attributes
                    .set_foreground(ColorAttribute::PaletteIndex(8))
                    .set_background(ColorAttribute::Default);
            };

            if let Some(index) = self.list_state.index() {
                if !self.typing && i == index {
                    attributes
                        .set_background(attributes.foreground())
                        .set_foreground(ColorAttribute::PaletteIndex(0));
                }
            }

            surface.draw_text(option, x + 1.0, y + i as f64 + 2.0, width * 0.5 - 3.0, attributes);
        }

        if let Some(selection) = self.list_state.selection() {
            let path = match self.text_state.text().ends_with('/') {
                true => Path::new(self.text_state.text()),
                false => match Path::new(self.text_state.text()).parent() {
                    Some(parent) => parent,
                    None => return,
                },
            }
            .join(selection);

            for (i, preview) in self.preview_state.sorted_options().iter().enumerate() {
                if i.saturating_add(3) > height as usize {
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
                    attributes,
                );
            }
        }

        if self.inner.focus && self.typing {
            state.cursor_position = (
                x + (self.text_state.grapheme_index() as f64).min(width.round() - 1.0),
                y,
            );
            state.cursor_color = style.caret_color();
            surface.add_change(Change::CursorVisibility(CursorVisibility::Visible));
        }
    }

    fn on_input_action(&mut self, _: &mut State, input_action: &InputAction) {
        match input_action {
            InputAction::Remove => {
                self.typing = true;
                self.text_state.backspace();
                self.sort_options();
            },
            InputAction::Submit | InputAction::Right => match self.typing {
                true => self.text_state.right(),
                false => {
                    if let Some(selection) = self.list_state.selection() {
                        let path = match self.text_state.text().ends_with('/') {
                            true => Path::new(self.text_state.text()),
                            false => match Path::new(self.text_state.text()).parent() {
                                Some(parent) => parent,
                                None => return,
                            },
                        }
                        .join(selection);

                        if let Some(path_str) = path.to_str() {
                            self.text_state.set_text(path_str);
                            if path.is_dir() {
                                self.text_state.character('/');
                            }

                            self.update_options();
                            self.update_preview();
                        }
                    }
                },
            },
            InputAction::Left => match self.typing {
                true => self.text_state.left(),
                false => {
                    if let Some(path_str) = PathBuf::new()
                        .join(self.text_state.text())
                        .parent()
                        .and_then(|path| path.to_str())
                    {
                        self.text_state.set_text(path_str);
                        self.text_state.character('/');
                        self.update_options();
                        self.update_preview();
                    }
                },
            },
            InputAction::Up => {
                if self.typing {
                    self.typing = false;
                    return;
                }

                if self.list_state.index().is_some() {
                    self.list_state.prev();
                    self.update_preview();
                }
            },
            InputAction::Down => {
                if self.typing {
                    self.typing = false;
                    return;
                }

                if self.list_state.index().is_some() {
                    self.list_state.next();
                    self.update_preview();
                }
            },
            InputAction::Delete => {
                self.typing = true;
                self.text_state.delete();
                self.sort_options();
            },
            InputAction::Insert(character) => {
                self.typing = true;
                self.text_state.character(*character);
                self.sort_options();
            },
            InputAction::Paste(clipboard) => {
                self.typing = true;
                self.text_state.paste(clipboard);
                self.sort_options();
            },
            _ => (),
        }
    }

    fn on_mouse_action(&mut self, state: &mut State, mouse_action: &MouseAction, x: f64, y: f64, _: f64, _: f64) {
        if self.inner.focus {
            let index = (mouse_action.y - y).round() as usize;
            if index == 0 {
                if mouse_action.just_pressed {
                    self.text_state.on_mouse_action(mouse_action, x);
                    self.typing = true;
                }
            } else if index > 1 && index.saturating_sub(2) < 6 {
                // workaround, probably should add hover state
                self.typing = false;

                self.list_state.set_index(index.saturating_sub(2));

                if mouse_action.just_pressed {
                    self.on_input_action(state, &InputAction::Right);
                }

                self.update_preview()
            }
        }
    }

    fn on_focus(&mut self, state: &mut State, focus: bool) {
        self.inner.focus = focus;

        match focus {
            true => {
                self.typing = true;
                self.update_options();
                self.update_preview();
            },
            false => {
                self.list_state.clear();
                self.preview_state.clear();

                state
                    .event_buffer
                    .push(Event::FilePicker(FilePickerEvent::FilePathChanged {
                        id: self.inner.id.to_owned(),
                        path: Path::new(self.text_state.text()).to_path_buf(),
                    }));
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
                    false => 8.0,
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
