use std::path::PathBuf;

use fig_proto::fig::FilePath;

use crate::prelude::*;

#[derive(Default)]
pub struct State {}

pub fn resolve_path(path: FilePath) -> Result<PathBuf> {
    todo!()
}

pub async fn read_file(path: &PathBuf) -> Result<Vec<u8>> {
    todo!()
}
