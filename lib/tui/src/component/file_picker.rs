use std::ffi::OsString;
use std::path::PathBuf;

use newton::{
    Color,
    DisplayState,
};

use crate::input::InputAction;
use crate::Style;

pub struct FilePicker {
    path: PathBuf,
    files: bool,
    folders: bool,
    extensions: Vec<String>,
    options: Vec<OsString>,
    preview: Vec<OsString>,
    focused: bool,
    index: usize,
    index_offset: usize,
    signal: Box<dyn Fn(PathBuf)>,
}

const MAX_ROWS: i32 = 8;

impl FilePicker {
    pub fn new(
        working_directory: impl Into<PathBuf>,
        files: bool,
        folders: bool,
        extensions: Vec<String>,
        signal: impl Fn(PathBuf) + 'static,
    ) -> Self {
        let working_directory = working_directory.into();

        Self {
            path: working_directory,
            files,
            folders,
            extensions,
            options: vec![],
            preview: vec![],
            focused: false,
            index: 0,
            index_offset: 0,
            signal: Box::new(signal),
        }
    }

    pub(crate) fn initialize(&mut self, width: &mut i32, height: &mut i32) {
        *width = 120;
        *height = 1;

        self.update_options();
        self.update_preview();
    }

    pub(crate) fn draw(&self, renderer: &mut DisplayState, style: &Style, x: i32, y: i32, width: i32, height: i32) {
        if height <= 0 || width <= 0 {
            return;
        }

        let (width, height) = match (usize::try_from(width), usize::try_from(height)) {
            (Ok(width), Ok(height)) => (width, height),
            _ => return,
        };

        let path = self.path.to_string_lossy();
        renderer.draw_string(
            &path[0..path.len().min(width)],
            x,
            y,
            style.color(),
            style.background_color(),
            false,
        );

        if height > 1 {
            renderer.draw_rect(
                '─',
                x,
                y + 1,
                width as i32,
                1,
                Color::DarkGrey,
                style.background_color(),
            );
            renderer.draw_symbol(
                '┬',
                x + width as i32 / 2 - 1,
                y + 1,
                Color::DarkGrey,
                style.background_color(),
                false,
            );
        }
        renderer.draw_rect(
            '│',
            x + width as i32 / 2 - 1,
            y + 2,
            1,
            height as i32 - 2,
            Color::DarkGrey,
            style.background_color(),
        );

        for (i, option) in self.options[self.index_offset
            ..self
                .options
                .len()
                .min(self.index_offset + usize::try_from(MAX_ROWS).unwrap())]
            .iter()
            .enumerate()
        {
            if i + 3 > height {
                break;
            }

            let path = self.path.join(option);
            let (mut color, mut background_color) = match path.is_dir() {
                true => (style.color(), style.background_color()),
                false => (Color::DarkGrey, Color::Reset),
            };

            if i == self.index - self.index_offset.min(self.index) {
                background_color = color;
                color = Color::Black;
            }

            let option = option.to_string_lossy();
            renderer.draw_string(
                &option[0..option.len().min(width / 2 - 3.min(width / 2))],
                x + 2,
                y + i32::try_from(i).unwrap() + 2,
                color,
                background_color,
                false,
            );
        }

        if let Some(option) = self.options.get(self.index) {
            for (i, preview) in self.preview.iter().enumerate() {
                if i + 3 > height {
                    break;
                }

                let path = self.path.join(option).join(preview);
                let (color, background_color) = match path.is_dir() {
                    true => (style.color(), style.background_color()),
                    false => (Color::DarkGrey, Color::Reset),
                };

                let preview = preview.to_string_lossy();
                renderer.draw_string(
                    &preview[0..preview.len().min(width / 2 - 1.min(width / 2))],
                    x + 2 + width as i32 / 2,
                    y + i32::try_from(i).unwrap() + 2,
                    color,
                    background_color,
                    false,
                );
            }
        }
    }

    pub(crate) fn on_input_action(&mut self, height: &mut i32, input: InputAction) -> bool {
        match input {
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
                    self.index = 0;
                    self.index_offset = 0;

                    if let InputAction::Submit = input {
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

                    *height = 1 + MAX_ROWS.min(i32::try_from(self.options.len()).unwrap());
                    if !self.options.is_empty() {
                        *height += 1;
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

                *height = 1 + MAX_ROWS.min(i32::try_from(self.options.len()).unwrap());
                if !self.options.is_empty() {
                    *height += 1;
                }
            },
            _ => (),
        }

        true
    }

    pub(crate) fn on_focus(&mut self, height: &mut i32, focused: bool) {
        match focused {
            true => {
                self.update_options();
                self.update_preview();

                *height = 1 + MAX_ROWS.min(i32::try_from(self.options.len()).unwrap());
            },
            false => {
                self.index = 0;
                self.index_offset = 0;

                self.options.clear();
                *height = 1;

                (self.signal)(self.path.clone());
            },
        }
        self.focused = focused;
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
