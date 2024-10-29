use std::path::Path;

use dbus::gnome_shell::ShellExtensions;
use fig_integrations::Integration;
use fig_integrations::desktop_entry::{
    AutostartIntegration,
    DesktopEntryIntegration,
};
use fig_integrations::gnome_extension::GnomeExtensionIntegration;
use fig_os_shim::Context;
use fig_util::CLI_BINARY_NAME;
use fig_util::directories::{
    fig_data_dir_ctx,
    local_webview_data_dir,
};
use tokio::sync::mpsc::Sender;
use tracing::warn;
use url::Url;

use crate::download::download_file;
use crate::index::UpdatePackage;
use crate::{
    Error,
    UpdateStatus,
};

macro_rules! bail {
    ($($arg:tt)*) => {
        return Err(Error::UpdateFailed(format!($($arg)*)))
    };
}

#[derive(Debug, PartialEq, Eq)]
struct ArchiveParser<'a> {
    /// The name of the archive file, e.g. `foo-x86_64-linux.tar.zst`
    file_name: &'a str,
    /// The prefix of the archive file, e.g. `foo-x86_64-linux`
    file_prefix: &'a str,
    /// The name of the binary, e.g. `foo`
    name: &'a str,
    /// The architecture of the binary, e.g. `x86_64`
    arch: Option<&'a str>,
    /// The operating system of the binary, e.g. `linux`
    os: Option<&'a str>,
}

impl<'a> ArchiveParser<'a> {
    fn from_url(url: &'a Url) -> Result<Self, Error> {
        let url = url.path();

        let Some((_, file_name)) = url.rsplit_once('/') else {
            return Err(Error::UpdateFailed(format!("Invalid archive URL: {url}")));
        };

        let Some((file_prefix, _)) = file_name.split_once('.') else {
            return Err(Error::UpdateFailed(format!("Invalid archive name: {file_name}")));
        };

        let mut components = file_prefix.split('-');

        let Some(name) = components.next() else {
            return Err(Error::UpdateFailed(format!("Invalid archive name: {file_name}")));
        };

        let arch = components.next();
        let os = components.next();

        Ok(Self {
            file_name,
            file_prefix,
            name,
            arch,
            os,
        })
    }
}

fn extract_archive(archive_path: &Path, tempdir: &Path) -> Result<(), Error> {
    let file = std::fs::File::open(archive_path)?;
    let mut decoder = zstd::Decoder::new(file)?;
    let mut tar = tar::Archive::new(&mut decoder);
    tar.unpack(tempdir)?;
    Ok(())
}

/// find binaries in the `bin_dir`` and move them to `$HOME/.local/bin`
async fn replace_bins(bin_dir: &Path) -> Result<(), Error> {
    let local_bin = fig_util::directories::home_local_bin()?;

    let mut res = Ok(());

    let mut read_bin_dir = tokio::fs::read_dir(bin_dir).await?;
    while let Ok(Some(bin)) = read_bin_dir.next_entry().await {
        let installed_bin_path = local_bin.join(bin.file_name());

        let _ = tokio::fs::remove_file(&installed_bin_path).await;
        if let Err(err) = tokio::fs::copy(bin.path(), installed_bin_path).await {
            if res.is_ok() {
                res = Err(err.into());
            }
        }
    }
    res
}

pub(crate) async fn update(
    UpdatePackage {
        download_url,
        sha256,
        size,
        ..
    }: UpdatePackage,
    tx: Sender<UpdateStatus>,
    _interactive: bool,
    _relaunch_dashboard: bool,
) -> Result<(), Error> {
    // check if the current exe can update
    let exe_path = std::env::current_exe()?.canonicalize()?;
    let Some(exe_name) = exe_path.file_name().and_then(|s| s.to_str()) else {
        bail!("Failed to get name of current executable: {exe_path:?}")
    };
    let Some(exe_parent) = exe_path.parent() else {
        bail!("Failed to get parent of current executable: {exe_path:?}")
    };
    // canonicalize to handle if the home dir is a symlink (like on Dev Desktops)
    let local_bin = fig_util::directories::home_local_bin()?.canonicalize()?;

    if exe_parent != local_bin {
        bail!(
            "Update is only supported for binaries installed in {local_bin:?}, the current executable is in {exe_parent:?}"
        );
    }

    if exe_name != CLI_BINARY_NAME {
        bail!("Update is only supported for {CLI_BINARY_NAME:?}, the current executable is {exe_name:?}");
    }

    let tempdir = tempfile::tempdir()?;

    let archive = ArchiveParser::from_url(&download_url)?;

    let archive_path = tempdir.path().join(archive.file_name);

    let real_hash = download_file(download_url.clone(), &archive_path, size, Some(tx.clone())).await?;
    if sha256 != real_hash {
        return Err(Error::UpdateFailed(format!(
            "Hash mismatch for {}: expected {sha256}, got {real_hash}",
            archive.file_name
        )));
    }

    let tempdir_path = tempdir.path().to_owned();
    tokio::task::spawn_blocking(move || extract_archive(&archive_path, &tempdir_path))
        .await
        .map_err(|err| Error::UpdateFailed(format!("Failed to extract {}: {err}", archive.file_name)))??;

    let bin_dir = tempdir.path().join(archive.name).join("bin");
    replace_bins(&bin_dir).await?;

    Ok(())
}

pub(crate) async fn uninstall_gnome_extension(
    ctx: &Context,
    shell_extensions: &ShellExtensions<Context>,
) -> Result<(), Error> {
    Ok(
        GnomeExtensionIntegration::new(ctx, shell_extensions, None::<&str>, None)
            .uninstall()
            .await?,
    )
}

pub(crate) async fn uninstall_desktop_entries(ctx: &Context) -> Result<(), Error> {
    DesktopEntryIntegration::new(ctx, None::<&str>, None, None)
        .uninstall()
        .await?;
    Ok(AutostartIntegration::uninstall(ctx).await?)
}

pub(crate) async fn uninstall_desktop(ctx: &Context) -> Result<(), Error> {
    let fs = ctx.fs();
    let data_dir_path = fig_data_dir_ctx(fs)?;
    if fs.exists(&data_dir_path) {
        fs.remove_dir_all(&data_dir_path)
            .await
            .map_err(|err| warn!(?err, "Failed to remove data dir"))
            .ok();
    }
    let webview_dir_path = local_webview_data_dir(ctx)?;
    if fs.exists(&webview_dir_path) {
        fs.remove_dir_all(&webview_dir_path)
            .await
            .map_err(|err| warn!(?err, "Failed to remove webview data dir"))
            .ok();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use fig_os_shim::Os;
    use fig_util::CLI_BINARY_NAME;
    use fig_util::directories::home_dir;

    use super::*;

    #[test]
    fn test_archive_def_from_url() {
        let url = Url::parse("https://example.com/path/q-x86_64-linux.tar.zst").unwrap();
        let archive_name = ArchiveParser::from_url(&url).unwrap();
        assert_eq!(archive_name, ArchiveParser {
            file_name: "q-x86_64-linux.tar.zst",
            file_prefix: "q-x86_64-linux",
            name: "q",
            arch: Some("x86_64"),
            os: Some("linux"),
        });
    }

    fn print_tree(p: &Path) {
        for entry in std::fs::read_dir(p).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let metadata = std::fs::metadata(&path).unwrap();
            if metadata.is_dir() {
                print_tree(&path);
            } else {
                println!("{}", path.display());
            }
        }
    }

    #[tokio::test]
    #[ignore = "needs archive to test with"]
    async fn test_extract_archive() {
        let archive_path = home_dir().unwrap().join(format!("{CLI_BINARY_NAME}.tar.zst"));
        let tempdir = tempfile::tempdir().unwrap();
        let tempdir_path = tempdir.path();

        extract_archive(&archive_path, tempdir.path()).unwrap();
        print_tree(tempdir_path);

        let bin_dir = tempdir.path().join(CLI_BINARY_NAME).join("bin");
        replace_bins(&bin_dir).await.unwrap();
    }

    #[tokio::test]
    async fn test_uninstall_desktop_removes_data_dir() {
        let ctx = Context::builder()
            .with_test_home()
            .await
            .unwrap()
            .with_os(Os::Linux)
            .build_fake();
        let fs = ctx.fs();
        let data_dir_path = fig_data_dir_ctx(fs).unwrap();
        fs.create_dir_all(&data_dir_path).await.unwrap();

        uninstall_desktop(&ctx).await.unwrap();

        assert!(!fs.exists(&data_dir_path));
    }
}
