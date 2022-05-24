use std::borrow::Cow;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use fig_settings::settings;
use parking_lot::Mutex;
use tracing::{
    error,
    info,
    warn,
};

static SELECTED_THEME: Mutex<Cow<str>> = parking_lot::const_mutex(Cow::Borrowed("hicolor"));

pub fn init() -> Result<()> {
    let mut use_local = true;

    if let Some(theme) = settings::get_string("autocomplete.iconTheme")? {
        if theme != "system" {
            use_local = set_theme(theme).is_err();
        }
    }

    if use_local {
        // attempt to get icon theme from gsettings
        if let Ok(output) = Command::new("gsettings")
            .arg("get")
            .arg("org.gnome.desktop.interface")
            .arg("icon-theme")
            .output()
        {
            if let Ok(output) = String::from_utf8(output.stdout) {
                // TODO(mia): ask someone to rewrite this is a more ideomatic way
                let _ = set_theme(output.split_at(1).1.split_at(output.len() - 3).0.to_string());
            }
        }
    }

    info!("selected theme {}", get_theme());

    Ok(())
}

fn set_theme(theme: String) -> Result<()> {
    if freedesktop_icons::list_themes().contains(&theme.as_str()) || theme == "hicolor" {
        *SELECTED_THEME.lock() = Cow::Owned(theme);
        Ok(())
    } else {
        warn!("invalid theme: {theme}");
        Err(anyhow::anyhow!("Invalid theme"))
    }
}

fn get_theme() -> String {
    SELECTED_THEME.lock().to_string()
}

pub fn lookup(name: &str) -> Option<Vec<u8>> {
    freedesktop_icons::lookup(name)
        .with_theme(&get_theme())
        .with_cache()
        .with_size(32)
        .find()
        .and_then(|path| match std::fs::read(&path) {
            Ok(s) => Some(s),
            Err(err) => {
                error!("failed reading icon at {path:?}: {err:?}");
                None
            },
        })
}
