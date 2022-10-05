use crate::index::UpdatePackage;
use crate::Error;

pub(crate) async fn update(_package: UpdatePackage, _deprecated: bool) -> Result<(), Error> {
    Err(Error::PackageManaged)
}
