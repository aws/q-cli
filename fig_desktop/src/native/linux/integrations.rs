use std::sync::atomic::Ordering;

use anyhow::{
    anyhow,
    Result,
};
use fig_proto::local::FocusedWindowDataHook;
use hashbrown::HashSet;
use once_cell::sync::Lazy;
use tracing::log::debug;

use super::WM_REVICED_DATA;
use crate::event::{
    Event,
    WindowEvent,
};
use crate::{
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

pub static WM_CLASS_WHITELIST: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    fig_util::terminal::LINUX_TERMINALS
        .iter()
        .filter_map(|t| t.wm_class())
        .collect()
});

static GSE_WHITELIST: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    fig_util::terminal::LINUX_TERMINALS
        .iter()
        .filter_map(|t| t.gnome_id())
        .collect()
});

fn from_source(from: &str) -> Option<&HashSet<&'static str>> {
    match from {
        "wm_class" => Some(&WM_CLASS_WHITELIST),
        "gse" => Some(&GSE_WHITELIST),
        _ => None,
    }
}

pub fn from_hook(hook: FocusedWindowDataHook, proxy: &EventLoopProxy) -> Result<()> {
    WM_REVICED_DATA.store(true, Ordering::Relaxed);

    if hook.hide() {
        proxy.send_event(Event::WindowEvent {
            window_id: AUTOCOMPLETE_ID,
            window_event: WindowEvent::Hide,
        })?;
        return Ok(());
    }

    debug!("focus event on {} from {}", hook.id, hook.source);
    if !from_source(&hook.source)
        .ok_or_else(|| anyhow!("received invalid focus window data source"))?
        .contains(hook.id.as_str())
    {
        proxy.send_event(Event::WindowEvent {
            window_id: AUTOCOMPLETE_ID,
            window_event: WindowEvent::Hide,
        })?;
    }

    Ok(())
}
