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
        let idx = floor_char_boundary(&from, len - 1);
        from.drain(idx..);
        from.insert(idx, '…');
    }
    from
}

// shamelessly stolen from the unstable `String::floor_char_boundary` function
pub fn floor_char_boundary(string: &String, index: usize) -> usize {
    if index >= string.len() {
        string.len()
    } else {
        let lower_bound = index.saturating_sub(3);
        let new_index = string.as_bytes()[lower_bound..=index]
            .iter()
            .rposition(|b| (*b as i8) >= -0x40);

        // SAFETY: we know that the character boundary will be within four bytes
        unsafe { lower_bound + new_index.unwrap_unchecked() }
    }
}
