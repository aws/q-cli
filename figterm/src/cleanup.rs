use anyhow::Result;
use fig_util::directories;
use tracing::trace;

pub fn cleanup() -> Result<()> {
    if let Ok(parent) = std::env::var("FIG_PARENT") {
        if !parent.is_empty() {
            trace!("Cleaning up parent file");
            let parent_path = directories::parent_socket_path(&parent)?;
            if parent_path.exists() {
                std::fs::remove_file(parent_path)?;
            }
        }
    }

    Ok(())
}
