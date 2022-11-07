use std::borrow::Cow;

use camino::{
    Utf8Path,
    Utf8PathBuf,
};
use fig_proto::fig::FilePath;

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
