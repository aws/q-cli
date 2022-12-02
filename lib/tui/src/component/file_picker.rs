use std::ffi::OsString;
use std::path::PathBuf;

use termwiz::color::ColorAttribute;
use termwiz::surface::Surface;

use super::ComponentData;
use crate::event_loop::{
    Event,
    State,
};
use crate::input::InputAction;
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
    path: PathBuf,
    files: bool,
    folders: bool,
    extensions: Vec<String>,
    options: Vec<OsString>,
    preview: Vec<OsString>,
    index: usize,
    index_offset: usize,
    inner: ComponentData,
}

impl FilePicker {
    pub fn new(
        id: impl ToString,
        path: impl Into<PathBuf>,
        files: bool,
        folders: bool,
        extensions: Vec<String>,
    ) -> Self {
        let path = path.into();

        Self {
            path,
            files,
            folders,
            extensions,
            options: vec![],
            preview: vec![],
            index: 0,
            index_offset: 0,
            inner: ComponentData::new(id.to_string(), true),
        }
    }

    fn update_options(&mut self) {
        self.options.clear();
        if let Ok(dir) = std::fs::read_dir(&self.path) {
            for file in dir.flatten() {
                self.options.push(file.file_name())
            }
        }

        self.options.sort_by(|a, b| {
            let apath = self.path.join(a);
            let bpath = self.path.join(b);

            match (apath.is_dir(), bpath.is_dir()) {
                (true, true) => a.cmp(b),
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (false, false) => a.cmp(b),
            }
        })
    }

    fn update_preview(&mut self) {
        self.preview.clear();
        if let Some(option) = self.options.get(self.index) {
            let path = self.path.join(option);
            if let Ok(dir) = std::fs::read_dir(&path) {
                for file in dir.flatten() {
                    self.preview.push(file.file_name());
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
    fn initialize(&mut self, _: &mut State) {
        self.inner.width = 120.0;
        self.inner.height = 1.0;

        self.update_options();
        self.update_preview();
    }

    fn draw(&self, state: &mut State, surface: &mut Surface, x: f64, y: f64, width: f64, height: f64, _: f64, _: f64) {
        if height <= 0.0 || width <= 0.0 {
            return;
        }

        let style = self.style(state);

        let path = self.path.to_string_lossy();
        surface.draw_text(&path, x, y, width, style.attributes());

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

            let path = self.path.join(option);
            let mut attributes = style.attributes();
            if !path.is_dir() {
                attributes
                    .set_foreground(ColorAttribute::PaletteIndex(8))
                    .set_background(ColorAttribute::Default);
            };

            if i == self.index - self.index_offset.min(self.index) {
                attributes
                    .set_background(attributes.foreground())
                    .set_foreground(ColorAttribute::PaletteIndex(0));
            }

            let option = option.to_string_lossy();
            surface.draw_text(&option, x + 2.0, y + i as f64 + 2.0, width / 2.0 - 3.0, attributes);
        }

        if let Some(option) = self.options.get(self.index) {
            for (i, preview) in self.preview.iter().enumerate() {
                if i + 3 > height as usize {
                    break;
                }

                let path = self.path.join(option);
                let mut attributes = style.attributes();
                if !path.is_dir() {
                    attributes
                        .set_foreground(ColorAttribute::PaletteIndex(8))
                        .set_background(ColorAttribute::Default);
                };

                let preview = preview.to_string_lossy();
                surface.draw_text(
                    &preview,
                    x + 2.0 + width * 0.5,
                    y + i as f64 + 2.0,
                    width / 2.0 - 1.0,
                    style.attributes(),
                );
            }
        }
    }

    fn on_input_action(&mut self, state: &mut State, input_action: InputAction) -> bool {
        match input_action {
            InputAction::Up => {
                if !self.options.is_empty() {
                    if self.index == 0 {
                        self.index_offset =
                            self.options.len() - usize::try_from(MAX_ROWS - 1).unwrap().min(self.options.len());
                    } else if self.index == self.index_offset {
                        self.index_offset -= 1;
                    }

                    self.index = (self.index + self.options.len() - 1) % self.options.len();

                    self.update_preview();
                }
            },
            InputAction::Down => {
                if !self.options.is_empty() {
                    if self.index == self.options.len() - 1 {
                        self.index_offset = 0;
                    } else if self.index == self.index_offset + usize::try_from(MAX_ROWS - 2).unwrap() {
                        self.index_offset += 1;
                    }

                    self.index = (self.index + 1) % self.options.len();

                    self.update_preview();
                }
            },
            InputAction::Submit | InputAction::Right => {
                if !self.options.is_empty() {
                    self.path.push(&self.options[self.index]);
                    state
                        .event_buffer
                        .push(Event::FilePicker(FilePickerEvent::FilePathChanged {
                            id: self.inner.id.to_owned(),
                            path: self.path.to_owned(),
                        }));

                    self.index = 0;
                    self.index_offset = 0;

                    if let InputAction::Submit = input_action {
                        if self.files {
                            if let Some(extension) = self.path.extension() {
                                if let Some(extension) = extension.to_str() {
                                    if self.extensions.contains(&extension.to_owned()) {
                                        return true;
                                    }
                                }
                            }
                        }

                        if self.folders && self.path.is_dir() {
                            return true;
                        }
                    }

                    self.update_options();
                    self.update_preview();

                    self.inner.height = 1.0 + MAX_ROWS.min(i32::try_from(self.options.len()).unwrap()) as f64;
                    if !self.options.is_empty() {
                        self.inner.height += 1.0;
                    }

                    return false;
                }
            },
            InputAction::Remove | InputAction::Left => {
                self.path.pop();

                self.index = 0;
                self.index_offset = 0;

                self.update_options();
                self.update_preview();

                self.inner.height = 1.0 + MAX_ROWS.min(i32::try_from(self.options.len()).unwrap()) as f64;
                if !self.options.is_empty() {
                    self.inner.height += 1.0;
                }
            },
            _ => (),
        }

        true
    }

    fn on_focus(&mut self, state: &mut State, focus: bool) {
        self.inner.focus = focus;

        match focus {
            true => {
                self.update_options();
                self.update_preview();

                self.inner.height = 1.0 + MAX_ROWS.min(i32::try_from(self.options.len()).unwrap()) as f64;
            },
            false => {
                self.index = 0;
                self.index_offset = 0;

                self.options.clear();
                self.inner.height = 1.0;

                state
                    .event_buffer
                    .push(Event::FilePicker(FilePickerEvent::FilePathChanged {
                        id: self.inner.id.to_owned(),
                        path: self.path.to_owned(),
                    }))
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
