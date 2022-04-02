use std::path::PathBuf;

use fig_proto::fig::FilePath;

use crate::prelude::*;

#[derive(Default)]
pub struct State {
    window_id: u32,
    process_id: u32,
}

pub fn resolve_path(path: FilePath) -> Result<PathBuf> {
    todo!()
}

pub async fn read_file(path: &PathBuf) -> Result<Vec<u8>> {
    todo!()
}
