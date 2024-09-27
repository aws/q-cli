use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{
    Path,
    PathBuf,
};
use std::str::FromStr;

use async_trait::async_trait;
use fig_os_shim::{
    EnvProvider,
    Fs,
    FsProvider,
};
use fig_util::consts::linux::DESKTOP_ENTRY_NAME;
use fig_util::consts::APP_PROCESS_NAME;
use fig_util::directories::home_dir_ctx;
use fig_util::PRODUCT_NAME;

use crate::error::{
    Error,
    ErrorExt,
    Result,
};
use crate::Integration;

/// Path to the local [PRODUCT_NAME] desktop entry.
pub fn local_entry_path<Ctx: FsProvider + EnvProvider>(ctx: &Ctx) -> Result<PathBuf> {
    Ok(home_dir_ctx(ctx)?.join(format!(".local/share/applications/{}", DESKTOP_ENTRY_NAME)))
}

/// Path to the local autostart symlink.
pub fn local_autostart_path<Ctx: FsProvider + EnvProvider>(ctx: &Ctx) -> Result<PathBuf> {
    Ok(home_dir_ctx(ctx)?.join(format!(".config/autostart/{}", DESKTOP_ENTRY_NAME)))
}

/// Path to the icon referenced by the desktop entry.
pub fn local_icon_path<Ctx: FsProvider>(ctx: &Ctx) -> Result<PathBuf> {
    Ok(fig_util::directories::fig_data_dir_ctx(ctx)?.join(format!("{APP_PROCESS_NAME}.png")))
}

/// Helper to create the parent directory of `path` if it doesn't already exist.
async fn create_parent(fs: &Fs, path: impl AsRef<Path>) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        if !parent.is_dir() {
            fs.fs().create_dir_all(parent).await?;
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct DesktopEntryIntegration<'a, Ctx> {
    ctx: &'a Ctx,

    /// Path to the desktop entry file to be installed.
    entry_path: PathBuf,

    /// Path to the desktop entry icon image to be installed.
    icon_path: PathBuf,

    /// Path to the executable to be set for the "Exec" field.
    exec_path: PathBuf,
}

impl<'a, Ctx> DesktopEntryIntegration<'a, Ctx>
where
    Ctx: FsProvider + EnvProvider,
{
    /// Creates a new [`DesktopEntryIntegration`].
    pub fn new<P>(ctx: &'a Ctx, entry_path: P, icon_path: P, exec_path: P) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            ctx,
            entry_path: entry_path.as_ref().into(),
            icon_path: icon_path.as_ref().into(),
            exec_path: exec_path.as_ref().into(),
        }
    }

    fn validate_field_path(entry_contents: &EntryContents, field: &str, expected_path: &PathBuf) -> Result<()> {
        match entry_contents.get_field(field) {
            Some(path) => {
                let set_path = PathBuf::from_str(path)
                    .map_err(|err| Error::ImproperInstallation(format!("Invalid field {}: {:?}", field, err).into()))?;
                if set_path != *expected_path {
                    return Err(Error::ImproperInstallation(
                        format!(
                            "Invalid field: {}. Expected {}, found {}",
                            field,
                            expected_path.to_string_lossy(),
                            set_path.to_string_lossy(),
                        )
                        .into(),
                    ));
                }
            },
            None => {
                return Err(Error::ImproperInstallation(
                    format!("Field {} is missing", field).into(),
                ));
            },
        }
        Ok(())
    }
}

#[async_trait]
impl<Ctx> Integration for DesktopEntryIntegration<'_, Ctx>
where
    Ctx: FsProvider + EnvProvider + Sync,
{
    fn describe(&self) -> String {
        "Desktop Entry Integration".to_owned()
    }

    async fn install(&self) -> Result<()> {
        if self.is_installed().await.is_ok() {
            return Ok(());
        }

        let fs = self.ctx.fs();

        let to_entry_path = local_entry_path(self.ctx)?;
        let to_icon_path = local_icon_path(self.ctx)?;

        // Required in case the user doesn't already have the local directories on their filesystem.
        create_parent(fs, &to_entry_path).await?;
        create_parent(fs, &to_icon_path).await?;

        // Install to the user local paths.
        let mut entry_contents = EntryContents::from_path(fs, &self.entry_path).await?;
        entry_contents.set_field("Exec", &self.exec_path.to_string_lossy());
        entry_contents.set_field("Name", PRODUCT_NAME);
        entry_contents.set_field("Icon", &to_icon_path.to_string_lossy());
        if !fs.exists(&to_entry_path) {
            fs.write(&to_entry_path, entry_contents.to_string()).await?;
        }
        if !fs.exists(&to_icon_path) {
            fs.copy(&self.icon_path, &to_icon_path).await?;
        }

        Ok(())
    }

    async fn uninstall(&self) -> Result<()> {
        let fs = self.ctx.fs();
        let to_entry_path = local_entry_path(self.ctx)?;
        let to_icon_path = local_icon_path(self.ctx)?;
        if fs.exists(&to_entry_path) {
            fs.remove_file(&to_entry_path).await?;
        }
        if fs.exists(&to_icon_path) {
            fs.remove_file(&to_icon_path).await?;
        }
        Ok(())
    }

    async fn is_installed(&self) -> Result<()> {
        let fs = self.ctx.fs();
        let to_entry_path = local_entry_path(self.ctx)?;
        let to_icon_path = local_icon_path(self.ctx)?;

        // Check if the installed entry exists.
        let entry_contents = match fs.read_to_string(&to_entry_path).await.with_path(&to_entry_path) {
            Ok(contents) => contents,
            Err(Error::Io(err)) if err.kind() == ErrorKind::NotFound => {
                return Err(Error::FileDoesNotExist(to_entry_path.clone().into()));
            },
            Err(err) => return Err(err),
        };
        let entry_contents = EntryContents::new(entry_contents);

        if !fs.exists(&to_icon_path) {
            return Err(Error::FileDoesNotExist(to_icon_path.clone().into()));
        }

        Self::validate_field_path(&entry_contents, "Exec", &self.exec_path)?;
        Self::validate_field_path(&entry_contents, "Icon", &to_icon_path)?;

        Ok(())
    }
}

/// Helper struct for parsing and updating a desktop entry.
#[derive(Debug, Clone)]
pub struct EntryContents {
    /// The lines of a desktop entry stored in a vector.
    lines: Vec<String>,

    /// Map of a key name to the line in `lines`.
    fields: HashMap<String, usize>,
}

impl std::fmt::Display for EntryContents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lines.join("\n"))
    }
}

impl EntryContents {
    pub fn new(entry_buf: String) -> Self {
        let lines = entry_buf.lines().map(str::to_owned).collect::<Vec<_>>();
        let fields = lines
            .iter()
            .enumerate()
            .filter_map(|(i, line)| {
                if !line.starts_with(|c: char| c.is_ascii_uppercase()) {
                    return None;
                }
                if let Some(j) = line.find("=") {
                    let key = &line[..j];
                    Some((key.to_string(), i))
                } else {
                    None
                }
            })
            .collect();
        Self { lines, fields }
    }

    pub async fn from_path<Fs: FsProvider, P: AsRef<Path>>(fs: &Fs, path: P) -> Result<Self> {
        let contents = fs.fs().read_to_string(path).await?;
        Ok(Self::new(contents))
    }

    pub fn from_path_sync<Fs: FsProvider, P: AsRef<Path>>(fs: &Fs, path: P) -> Result<Self> {
        let contents = fs.fs().read_to_string_sync(path)?;
        Ok(Self::new(contents))
    }

    pub fn get_field(&self, key: &str) -> Option<&str> {
        self.fields
            .get(key)
            .and_then(|i| self.lines.get(*i))
            .and_then(|line| line.as_str().split_once("="))
            .map(|(_, v)| v)
    }

    pub fn set_field(&mut self, key: &str, value: &str) {
        let to_add = format!("{}={}", key, value);
        match self.fields.get(key) {
            Some(i) => self.lines[*i] = to_add,
            None => {
                self.lines.push(to_add);
                self.fields.insert(key.to_string(), self.lines.len() - 1);
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutostartIntegration<'a, Ctx> {
    ctx: &'a Ctx,
}

impl<'a, Ctx> AutostartIntegration<'a, Ctx>
where
    Ctx: FsProvider + EnvProvider,
{
    pub fn new(ctx: &'a Ctx) -> Self {
        Self { ctx }
    }
}

#[async_trait]
impl<Ctx> Integration for AutostartIntegration<'_, Ctx>
where
    Ctx: FsProvider + EnvProvider + Sync,
{
    fn describe(&self) -> String {
        "Desktop Autostart Entry Integration".to_owned()
    }

    async fn install(&self) -> Result<()> {
        if self.is_installed().await.is_ok() {
            return Ok(());
        }

        let fs = self.ctx.fs();
        let to_entry_path = local_entry_path(self.ctx)?;
        let to_autostart_path = local_autostart_path(self.ctx)?;
        create_parent(fs, &to_autostart_path).await?;
        fs.symlink(&to_entry_path, &to_autostart_path).await?;
        Ok(())
    }

    async fn uninstall(&self) -> Result<()> {
        let fs = self.ctx.fs();
        let to_autostart_path = local_autostart_path(self.ctx)?;
        if fs.exists(&to_autostart_path) {
            fs.remove_file(&to_autostart_path).await?;
        }
        Ok(())
    }

    async fn is_installed(&self) -> Result<()> {
        let fs = self.ctx.fs();
        let to_entry_path = local_entry_path(self.ctx)?;
        let to_autostart_path = local_autostart_path(self.ctx)?;
        if !fs.exists(&to_autostart_path) {
            return Err(Error::FileDoesNotExist(to_autostart_path.clone().into()));
        }
        let read_path = fs.read_link(&to_autostart_path).await?;
        if read_path != to_entry_path {
            Err(Error::ImproperInstallation(
                format!("Unexpected link path: {}", read_path.to_string_lossy()).into(),
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use fig_os_shim::{
        Context,
        ContextBuilder,
    };

    use super::*;

    const TEST_DESKTOP_ENTRY: &str = r#"[Desktop Entry]
Categories=Development;
Exec=q-desktop
Icon=q-desktop
Name=q_desktop
Terminal=false
Type=Application"#;

    const TEST_EXEC_VALUE: &str = "/app.appimage";

    #[tokio::test]
    async fn test_entry_contents() {
        let mut contents =
            EntryContents::new("[Desktop Entry]\n# Some Comment\nExec=testapp\nIcon=testapp.png".to_string());
        assert_eq!(contents.get_field("Exec"), Some("testapp"));
        assert_eq!(contents.get_field("Icon"), Some("testapp.png"));
        contents.set_field("Icon", "/path/img.png");
        assert_eq!(
            contents.to_string(),
            "[Desktop Entry]\n# Some Comment\nExec=testapp\nIcon=/path/img.png"
        );
    }

    async fn make_test_desktop_entry(ctx: &Context) -> DesktopEntryIntegration<'_, Context> {
        let fs = ctx.fs();
        fs.write("/app.desktop", TEST_DESKTOP_ENTRY).await.unwrap();
        fs.write("/app.png", "image").await.unwrap();
        DesktopEntryIntegration::new(ctx, "/app.desktop", "/app.png", TEST_EXEC_VALUE)
    }

    #[tokio::test]
    async fn test_desktop_entry_integration_install_and_uninstall() {
        let ctx = ContextBuilder::new().with_test_home().await.unwrap().build();
        let fs = ctx.fs();
        let integration = make_test_desktop_entry(&ctx).await;
        assert!(integration.is_installed().await.is_err());

        // Test install.
        integration.install().await.unwrap();

        // Validating it was installed.
        assert!(integration.is_installed().await.is_ok());
        let installed_entry_path = local_entry_path(&ctx).unwrap();
        let installed_icon_path = local_icon_path(ctx.fs()).unwrap();
        assert!(
            fs.exists(&installed_entry_path),
            "desktop entry should have been created"
        );
        assert_eq!(
            fs.read_to_string(&installed_icon_path).await.unwrap(),
            "image",
            "image should have been created"
        );

        // Validating the content of the desktop entry
        let entry_contents = EntryContents::from_path(fs, &installed_entry_path).await.unwrap();
        let actual_exec = entry_contents.get_field("Exec").unwrap();
        assert_eq!(actual_exec, TEST_EXEC_VALUE, "invalid Exec field");
        let actual_icon = entry_contents.get_field("Icon").unwrap();
        assert_eq!(actual_icon, installed_icon_path.to_string_lossy(), "invalid Icon field");

        // Test uninstall.
        integration.uninstall().await.unwrap();

        // Validating it was uninstalled.
        assert!(integration.is_installed().await.is_err());
        assert!(
            !fs.exists(installed_entry_path),
            "installed desktop entry should have been deleted"
        );
        assert!(
            !fs.exists(installed_icon_path),
            "installed icon should have been deleted"
        );
    }

    #[tokio::test]
    async fn test_autostart_integration_install_and_uninstall() {
        let ctx = ContextBuilder::new().with_test_home().await.unwrap().build();
        make_test_desktop_entry(&ctx).await.install().await.unwrap();
        let autostart = AutostartIntegration::new(&ctx);
        assert!(autostart.is_installed().await.is_err());

        // Test install.
        autostart.install().await.unwrap();
        autostart.is_installed().await.unwrap();
        assert!(autostart.is_installed().await.is_ok());
        let installed_entry_path = local_entry_path(&ctx).unwrap();
        let installed_autostart_path = local_autostart_path(&ctx).unwrap();
        assert_eq!(
            ctx.fs().read_link(&installed_autostart_path).await.unwrap(),
            installed_entry_path
        );

        // Test uninstall.
        autostart.uninstall().await.unwrap();
        assert!(autostart.is_installed().await.is_err());
        assert!(!ctx.fs().exists(&installed_autostart_path));
    }
}
