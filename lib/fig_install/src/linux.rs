use fig_util::manifest::manifest;
use tokio::sync::mpsc::Sender;

use crate::index::UpdatePackage;
use crate::{
    Error,
    UpdateStatus,
};

pub(crate) async fn update(
    _package: UpdatePackage,
    _tx: Sender<UpdateStatus>,
    _interactive: bool,
    _relaunch_dashboard: bool,
) -> Result<(), Error> {
    if manifest().is_none() {
        Err(Error::UpdateFailed("Please remove `~/.local/bin/fig` and reinstall Fig with `curl -fSsL https://fig.io/install-headless.sh | bash`".into()))
    } else {
        Err(Error::PackageManaged)
    }
}
