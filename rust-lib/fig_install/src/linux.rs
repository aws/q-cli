use fig_util::manifest::manifest;

use crate::index::UpdatePackage;
use crate::Error;

pub(crate) async fn update(_package: UpdatePackage, _deprecated: bool) -> Result<(), Error> {
    if manifest().is_none() {
        Err(Error::LegacyUpdateFailed("Please remove `~/.local/bin/fig` and reinstall Fig with `curl -fSsL https://fig.io/install-headless.sh | bash`".into()))
    } else {
        Err(Error::PackageManaged)
    }
}
