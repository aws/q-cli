use std::path::PathBuf;

use fig_proto::fig::FilePath;

pub fn resolve_filepath(file_path: FilePath) -> PathBuf {
    let convert = |path: String| {
        if file_path.expand_tilde_in_path {
            shellexpand::tilde(&path).into_owned()
        } else {
            path
        }
    };
    let mut relative_to = file_path
        .relative_to
        .map(convert)
        .map(PathBuf::from)
        .unwrap_or_else(PathBuf::new);
    let path = PathBuf::from(convert(file_path.path));
    relative_to.push(path);

    relative_to
}

pub fn build_filepath(path: PathBuf) -> FilePath {
    FilePath {
        path: path.to_string_lossy().to_string(),
        relative_to: None,
        expand_tilde_in_path: false,
    }
}

pub fn truncate_string(mut from: String, len: usize) -> String {
    if from.len() > len {
        from.drain(len..);
        from.insert(len, '…');
    }
    from
}
