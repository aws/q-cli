use std::borrow::Cow;

use camino::{
    Utf8Path,
    Utf8PathBuf,
};
use fig_proto::fig::FilePath;
use serde::{
    Deserialize,
    Serialize,
};

pub fn resolve_filepath<'a>(file_path: &'a FilePath) -> Cow<'a, Utf8Path> {
    let convert = |path: &'a str| -> Cow<str> {
        if file_path.expand_tilde_in_path() {
            shellexpand::tilde(path)
        } else {
            path.into()
        }
    };

    match file_path.relative_to {
        Some(ref relative_to) => Utf8Path::new(&convert(relative_to))
            .join(&*convert(&file_path.path))
            .into(),
        None => match convert(&file_path.path) {
            Cow::Borrowed(path) => Utf8Path::new(path).into(),
            Cow::Owned(path) => Utf8PathBuf::from(path).into(),
        },
    }
}

pub fn build_filepath(path: impl Into<String>) -> FilePath {
    FilePath {
        path: path.into(),
        relative_to: None,
        expand_tilde_in_path: Some(false),
    }
}

pub fn truncate_string(mut from: String, len: usize) -> String {
    if from.len() > len {
        let idx = floor_char_boundary(&from, len - 1);
        from.drain(idx..);
        from.insert(idx, '…');
    }
    from
}

// shamelessly stolen from the unstable `String::floor_char_boundary` function
pub fn floor_char_boundary(string: &str, index: usize) -> usize {
    if index >= string.len() {
        string.len()
    } else {
        let lower_bound = index.saturating_sub(3);
        let new_index = string.as_bytes()[lower_bound..=index]
            .iter()
            .rposition(|b| (*b as i8) >= -0x40);

        // we know that the character boundary will be within four bytes
        lower_bound + new_index.unwrap()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rect<U, V> {
    pub x: U,
    pub y: U,
    pub width: V,
    pub height: V,
}
