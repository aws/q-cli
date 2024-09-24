use std::collections::HashMap;
use std::path::{
    Path,
    PathBuf,
};
use std::sync::Arc;

use fig_os_shim::{
    Context,
    Env,
};
use thiserror::Error;

use crate::consts::APP_PROCESS_NAME;
use crate::consts::linux::DESKTOP_ENTRY_NAME;
use crate::{
    PRODUCT_NAME,
    directories,
};

#[derive(Debug, Error)]
pub enum DesktopError {
    #[error("missing home directory")]
    MissingHome,
    #[error("desktop entry is missing the \"Exec\" key")]
    MissingExecEntry,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    DirectoryError(#[from] directories::DirectoryError),
}

/// Path to the local [PRODUCT_NAME] desktop entry.
pub fn local_entry_path(env: &Env) -> Result<PathBuf, DesktopError> {
    Ok(env
        .home()
        .ok_or(DesktopError::MissingHome)?
        .join(format!(".local/share/applications/{DESKTOP_ENTRY_NAME}")))
}

/// Path to the local autostart symlink.
pub fn local_autostart_path(env: &Env) -> Result<PathBuf, DesktopError> {
    Ok(env
        .home()
        .ok_or(DesktopError::MissingHome)?
        .join(format!(".config/autostart/{DESKTOP_ENTRY_NAME}")))
}

/// Path to the icon referenced by the desktop entry.
pub fn local_icon_path(ctx: &Context) -> Result<PathBuf, DesktopError> {
    Ok(directories::fig_data_dir_ctx(ctx)?.join(format!("{APP_PROCESS_NAME}.png")))
}

/// Helper to create the parent directory of `path` if it doesn't already exist.
async fn create_parent(ctx: &Context, path: impl AsRef<Path>) -> Result<(), DesktopError> {
    if let Some(parent) = path.as_ref().parent() {
        if !parent.is_dir() {
            ctx.fs().create_dir_all(parent).await?;
        }
    }
    Ok(())
}

/// Represents the desktop entry file installed locally.
pub struct DesktopEntry {
    ctx: Arc<Context>,
    contents: EntryContents,
    entry_path: PathBuf,
    autostart_path: PathBuf,
}

impl DesktopEntry {
    /// Creates a new desktop entry by installing the desktop file provided by `entry_path` and
    /// the icon provided by `icon_path`.
    ///
    /// Note that the icon must be a png.
    pub async fn new<T: AsRef<Path>>(
        ctx: Arc<Context>,
        from_entry_path: T,
        from_icon_path: T,
    ) -> Result<Self, DesktopError> {
        let (fs, env) = (ctx.fs(), ctx.env());
        let to_entry_path = local_entry_path(env)?;
        let to_autostart_path = local_autostart_path(env)?;
        let to_icon_path = local_icon_path(&ctx)?;

        // Required in case the user doesn't already have the local directories on their filesystem.
        create_parent(&ctx, &to_entry_path).await?;
        create_parent(&ctx, &to_icon_path).await?;

        // Update the Icon field to match the local installed path.
        let mut contents = EntryContents::new(fs.read_to_string(&from_entry_path).await?);
        contents.set_field("Name", PRODUCT_NAME);
        contents.set_field("Icon", &to_icon_path.to_string_lossy());

        // Install to the user local paths.
        fs.write(&to_entry_path, contents.to_string()).await?;
        fs.copy(from_icon_path, &to_icon_path).await?;

        Ok(Self {
            ctx,
            contents,
            entry_path: to_entry_path,
            autostart_path: to_autostart_path,
        })
    }

    /// Creates a new [DesktopEntry], initializing from the currently installed desktop entry.
    pub fn new_existing(ctx: Arc<Context>) -> Result<Self, DesktopError> {
        let entry_path = local_entry_path(ctx.env())?;
        let autostart_path = local_autostart_path(ctx.env())?;
        let contents = ctx.fs().read_to_string_sync(&entry_path)?;
        Ok(Self {
            ctx,
            contents: EntryContents::new(contents),
            entry_path,
            autostart_path,
        })
    }

    pub async fn autostart_enabled(&self) -> Result<bool, DesktopError> {
        let fs = self.ctx.fs();
        if fs.exists(&self.autostart_path) {
            Ok(fs.read_link(&self.autostart_path).await? == fs.chroot_path(&self.entry_path))
        } else {
            Ok(false)
        }
    }

    pub async fn enable_autostart(&self) -> Result<(), DesktopError> {
        if self.autostart_enabled().await? {
            return Ok(());
        }
        create_parent(&self.ctx, &self.autostart_path).await?;
        self.ctx.fs().symlink(&self.entry_path, &self.autostart_path).await?;
        Ok(())
    }

    pub async fn disable_autostart(&self) -> Result<(), DesktopError> {
        match self.ctx.fs().remove_file(&self.autostart_path).await {
            Ok(_) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
    }

    pub fn get_field(&self, key: &str) -> Option<&str> {
        self.contents.get_field(key)
    }

    pub async fn set_field(&mut self, key: &str, value: &str) -> Result<(), DesktopError> {
        self.contents.set_field(key, value);
        self.ctx.fs().write(&self.entry_path, self.contents.to_string()).await?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct EntryContents {
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

#[cfg(test)]
mod tests {
    use fig_os_shim::ContextBuilder;

    use super::*;

    const TEST_DESKTOP_ENTRY: &str = r#"[Desktop Entry]
Categories=Development;
Exec=q-desktop
Icon=q-desktop
Name=q_desktop
Terminal=false
Type=Application"#;

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

    #[tokio::test]
    async fn test_new_desktop_entry() {
        let ctx = ContextBuilder::new().with_test_home().await.unwrap().build();
        let fs = ctx.fs();
        fs.write("/app.desktop", TEST_DESKTOP_ENTRY).await.unwrap();
        fs.write("/app.png", "image").await.unwrap();

        // When
        let desktop_entry = DesktopEntry::new(Arc::clone(&ctx), "/app.desktop", "/app.png")
            .await
            .unwrap();

        // Then
        assert!(fs.exists(local_entry_path(ctx.env()).unwrap()));
        assert_eq!(
            fs.read_to_string(local_icon_path(&ctx).unwrap()).await.unwrap(),
            "image"
        );
        assert_eq!(desktop_entry.get_field("Exec").unwrap(), "q-desktop");
        assert!(!desktop_entry.autostart_enabled().await.unwrap());
    }

    #[tokio::test]
    async fn test_desktop_entry_sets_field() {
        let ctx = ContextBuilder::new().with_test_home().await.unwrap().build();
        let fs = ctx.fs();
        fs.write(
            "/app.desktop",
            "[Desktop Entry]\n#Comment\nIcon=q-desktop\nExec=q-desktop",
        )
        .await
        .unwrap();
        fs.write("/app.png", "image").await.unwrap();

        // When
        let mut desktop_entry = DesktopEntry::new(Arc::clone(&ctx), "/app.desktop", "/app.png")
            .await
            .unwrap();
        desktop_entry.set_field("Icon", "/test-icon").await.unwrap();

        // Then
        assert_eq!(desktop_entry.get_field("Icon"), Some("/test-icon"));
    }

    #[tokio::test]
    async fn test_desktop_entry_enabling_and_disabling_autostart() {
        let ctx = ContextBuilder::new().with_test_home().await.unwrap().build();
        let fs = ctx.fs();
        fs.write("/app.desktop", TEST_DESKTOP_ENTRY).await.unwrap();
        fs.write("/app.png", "image").await.unwrap();

        let desktop_entry = DesktopEntry::new(Arc::clone(&ctx), "/app.desktop", "/app.png")
            .await
            .unwrap();

        // Enabling
        desktop_entry.enable_autostart().await.unwrap();
        assert_eq!(
            fs.read_link(&local_autostart_path(ctx.env()).unwrap()).await.unwrap(),
            fs.chroot_path(local_entry_path(ctx.env()).unwrap())
        );
        assert!(desktop_entry.autostart_enabled().await.unwrap());
        desktop_entry.enable_autostart().await.unwrap(); // enabling twice should not return error

        // Disabling
        desktop_entry.disable_autostart().await.unwrap();
        assert!(!fs.exists(local_autostart_path(ctx.env()).unwrap()));
        assert!(!desktop_entry.autostart_enabled().await.unwrap());
    }
}
