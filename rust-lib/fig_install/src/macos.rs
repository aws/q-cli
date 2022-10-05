// use crate::api::file::{ArchiveFormat, Extract, Move};

use fig_ipc::local::update_command;
use fig_util::launch_fig;

use crate::index::UpdatePackage;
use crate::Error;

pub(crate) async fn update(_package: UpdatePackage, deprecated: bool) -> Result<(), Error> {
    // Let desktop app handle updates on macOS
    launch_fig(true, false)?;

    if update_command(deprecated).await.is_err() {
        return Err(Error::LegacyUpdateFailed(
            "Unable to connect to Fig, it may not be running. To launch Fig, run 'fig launch'".to_owned(),
        ));
    }

    Ok(())
}
