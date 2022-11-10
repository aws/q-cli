use std::collections::HashMap;
use std::env::temp_dir;
use std::io::Cursor;
use std::path::PathBuf;

use async_trait::async_trait;
use macos_utils::url::path_for_application;
use serde::Deserialize;
use zip::ZipArchive;

use crate::error::Result;
use crate::{
    Error,
    Integration,
};

#[derive(Clone)]
pub struct IntelliJVariant {
    pub application_name: &'static str,
    bundle_identifier: &'static str,
    organization: &'static str,
}

impl IntelliJVariant {
    const fn jetbrains(application_name: &'static str, bundle_identifier: &'static str) -> IntelliJVariant {
        IntelliJVariant {
            application_name,
            bundle_identifier,
            organization: "JetBrains",
        }
    }
}

const PLUGIN_PREFIX: &str = "jetbrains-extension-";
const PLUGIN_SLUG: &str = "jetbrains-extension-2.0.0";
static PLUGIN_CONTENTS: &[u8] = include_bytes!("plugin.zip");

pub static VARIANTS: &[IntelliJVariant] = &[
    IntelliJVariant::jetbrains("JetBrains IDEA", "com.jetbrains.intellij"),
    IntelliJVariant::jetbrains("JetBrains IDEA CE", "com.jetbrains.intellij.ce"),
    IntelliJVariant::jetbrains("JetBrains WebStorm", "com.jetbrains.WebStorm"),
    IntelliJVariant::jetbrains("JetBrains GoLand", "com.jetbrains.goland"),
    IntelliJVariant::jetbrains("JetBrains PhpStorm", "com.jetbrains.PhpStorm"),
    IntelliJVariant::jetbrains("JetBrains PyCharm", "com.jetbrains.pycharm"),
    IntelliJVariant::jetbrains("JetBrains PyCharm CE", "com.jetbrains.pycharm.ce"),
    IntelliJVariant::jetbrains("JetBrains AppCode", "com.jetbrains.AppCode"),
    IntelliJVariant::jetbrains("JetBrains CLion", "com.jetbrains.CLion"),
    IntelliJVariant::jetbrains("JetBrains Rider", "com.jetbrains.rider"),
    IntelliJVariant::jetbrains("JetBrains RubyMine", "com.jetbrains.rubymine"),
    IntelliJVariant::jetbrains("JetBrains DataSpell", "com.jetbrains.dataspell"),
    IntelliJVariant {
        application_name: "Android Studio",
        bundle_identifier: "com.google.android.studio",
        organization: "Google",
    },
];

pub fn variants_installed() -> Vec<IntelliJVariant> {
    VARIANTS
        .iter()
        .filter(|variant| path_for_application(variant.bundle_identifier).is_some())
        .cloned()
        .collect()
}

pub struct IntelliJIntegration {
    pub variant: IntelliJVariant,
}

#[derive(Deserialize)]
struct InfoPList {
    #[serde(rename = "JVMOptions")]
    jvm_options: JVMOptions,
}

#[derive(Deserialize)]
struct JVMOptions {
    #[serde(rename = "Properties")]
    properties: HashMap<String, String>,
}

impl IntelliJIntegration {
    fn get_jvm_properties(&self) -> Result<HashMap<String, String>> {
        let plist_path = path_for_application(self.variant.bundle_identifier)
            .ok_or_else(|| Error::ApplicationNotInstalled(self.variant.application_name.into()))?
            .join("Contents/Info.plist");

        let contents: InfoPList = plist::from_file(plist_path)
            .map_err(|err| Error::Custom(format!("Could not read plist file: {err:?}").into()))?;

        Ok(contents.jvm_options.properties)
    }

    fn application_folder(&self) -> Result<PathBuf> {
        let mut props = self
            .get_jvm_properties()
            .map_err(|err| Error::Custom(format!("Couldn't get JVM properties: {err:?}").into()))?;

        let selector = props
            .remove("idea.paths.selector")
            .ok_or_else(|| Error::Custom("Could not read `idea.paths.selector` from jvm options".into()))?;

        Ok(dirs::data_local_dir()
            .ok_or_else(|| Error::Custom("Could not read application support directory".into()))?
            .join(self.variant.organization)
            .join(selector))
    }
}

#[async_trait]
impl Integration for IntelliJIntegration {
    fn describe(&self) -> String {
        format!("{} Integration", self.variant.application_name)
    }

    async fn install(&self) -> Result<()> {
        if self.is_installed().await.is_ok() {
            return Ok(());
        }

        let application_folder = self.application_folder()?;

        if !application_folder.exists() {
            return Err(Error::Custom("application folder does not exist".into()));
        }

        self.uninstall().await?;

        let plugins_folder = application_folder.join("plugins");
        let destination_folder = plugins_folder.join(PLUGIN_SLUG);

        if destination_folder.exists() {
            tokio::fs::remove_dir_all(&destination_folder).await.map_err(|err| {
                Error::Custom(format!("Failed removing destination folder {destination_folder:?}: {err:?}").into())
            })?;
        }

        let mut archive = ZipArchive::new(Cursor::new(PLUGIN_CONTENTS))
            .map_err(|err| Error::Custom(format!("Failed reading bundled plugin zip: {err:?}").into()))?;

        let tmp = temp_dir();

        archive.extract(&tmp)?;

        let tmp_plugin_path = tmp.join("jetbrains-extension");

        tokio::fs::rename(&tmp_plugin_path, &destination_folder)
            .await
            .map_err(|err| {
                Error::Custom(format!("Failed renaming extracted plugin path {tmp_plugin_path:?} to destination folder {destination_folder:?}: {err:?}").into())
            })?;

        Ok(())
    }

    async fn uninstall(&self) -> Result<()> {
        let plugins_folder = self.application_folder()?.join("plugins");

        let mut entries = tokio::fs::read_dir(&plugins_folder).await.map_err(|err| {
            Error::Custom(format!("Failed reading plugins folder dir {plugins_folder:?}: {err:?}").into())
        })?;
        while let Some(entry) = entries.next_entry().await.map_err(|err| {
            Error::Custom(format!("Failed reading next entry in plugins folder dir {plugins_folder:?}: {err:?}").into())
        })? {
            if entry.file_name().to_string_lossy().starts_with(PLUGIN_PREFIX) {
                tokio::fs::remove_dir_all(entry.path()).await.map_err(|err| {
                    Error::Custom(
                        format!(
                            "Failed removing entry {:?} from plugins folder dir {plugins_folder:?}: {err:?}",
                            entry.path()
                        )
                        .into(),
                    )
                })?;
            }
        }

        Ok(())
    }

    async fn is_installed(&self) -> Result<()> {
        let plugin_folder = self.application_folder()?.join("plugins").join(PLUGIN_SLUG);

        if !plugin_folder.exists() {
            return Err(Error::Custom("Plugin not installed".into()));
        }

        Ok(())
    }
}
