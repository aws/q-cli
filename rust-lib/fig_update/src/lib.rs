use anyhow::Result;
use cfg_if::cfg_if;
use fig_util::get_system_id;

pub mod index;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

pub use index::check as check_for_updates;
use index::Package;

fn system_threshold(version: &str) -> Result<u8> {
    let mut threshold: u8 = 0;

    let mut apply = |from: &str| {
        for ch in from.chars() {
            threshold = threshold.wrapping_add((((ch as u32) % 256) as u8).wrapping_add(128));
        }
    };

    // different for each system
    apply(&get_system_id()?);
    // different for each version
    apply(version);

    Ok(threshold)
}

#[allow(unused_variables)]
pub fn apply_update(package: Package) -> Result<()> {
    cfg_if! {
        if #[cfg(windows)] {
            return windows::update(package);
        } else if #[cfg(target_os = "macos")] {
            return macos::update(package),
        } else {
            return Err(anyhow::anyhow!("updates not supported on this platform"))
        }
    }
}
