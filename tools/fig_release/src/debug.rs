use std::path::PathBuf;

use fig_request::{
    Method,
    Request,
};
use serde::Serialize;

use crate::cli::DebugAction;

pub async fn debug(action: DebugAction) -> eyre::Result<()> {
    let ret = match action {
        DebugAction::GetIndexDirty => {
            Request::new_release(Method::GET, "/debug/index_dirty")
                .auth()
                .text()
                .await?
        },
        DebugAction::SetIndexDirty => {
            Request::new_release(Method::POST, "/debug/index_dirty")
                .auth()
                .text()
                .await?
        },
        DebugAction::GetSyncDirty => {
            Request::new_release(Method::GET, "/debug/sync_dirty")
                .auth()
                .text()
                .await?
        },
        DebugAction::SetSyncDirty => {
            Request::new_release(Method::POST, "/debug/sync_dirty")
                .auth()
                .text()
                .await?
        },
        DebugAction::ReadFile { path, base } => {
            Request::new_release(Method::GET, "/debug/file")
                .auth()
                .query(&GetFileArgs { path, base })
                .text()
                .await?
        },
    };

    println!("{ret}");

    Ok(())
}

#[derive(Serialize)]
struct GetFileArgs {
    path: PathBuf,
    base: bool,
}
