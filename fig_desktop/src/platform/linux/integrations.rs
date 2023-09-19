use std::collections::HashMap;
use std::sync::atomic::Ordering;

use anyhow::{
    anyhow,
    Result,
};
use fig_proto::local::FocusedWindowDataHook;
use fig_util::Terminal;
use once_cell::sync::Lazy;
use tracing::debug;

use super::WM_REVICED_DATA;
use crate::event::{
    Event,
    WindowEvent,
};
use crate::platform::{
    ActiveWindowData,
    PlatformState,
};
use crate::{
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

pub static WM_CLASS_WHITELIST: Lazy<HashMap<&'static str, Terminal>> = Lazy::new(|| {
    let mut whitelist = HashMap::new();
    for terminal in fig_util::terminal::LINUX_TERMINALS {
        if let Some(wm_class) = terminal.wm_class() {
            whitelist.insert(wm_class, terminal.clone());
        }
    }
    whitelist
});

pub static GSE_WHITELIST: Lazy<HashMap<&'static str, Terminal>> = Lazy::new(|| {
    let mut whitelist = HashMap::new();
    for terminal in fig_util::terminal::LINUX_TERMINALS {
        if let Some(gnome_id) = terminal.gnome_id() {
            whitelist.insert(gnome_id, terminal.clone());
        }
    }
    whitelist
});

fn from_source(from: &str) -> Option<&HashMap<&'static str, Terminal>> {
    match from {
        "wm_class" => Some(&WM_CLASS_WHITELIST),
        "gse" => Some(&GSE_WHITELIST),
        _ => None,
    }
}

pub fn from_hook(hook: FocusedWindowDataHook, platform_state: &PlatformState, proxy: &EventLoopProxy) -> Result<()> {
    WM_REVICED_DATA.store(true, Ordering::Relaxed);

    if hook.hide() {
        proxy.send_event(Event::WindowEvent {
            window_id: AUTOCOMPLETE_ID,
            window_event: WindowEvent::Hide,
        })?;
        return Ok(());
    }

    debug!("focus event on {} from {}", hook.id, hook.source);
    if from_source(&hook.source)
        .ok_or_else(|| anyhow!("received invalid focus window data source"))?
        .contains_key(hook.id.as_str())
    {
        let inner = hook.inner.unwrap();
        let outer = hook.outer.unwrap();
        let mut handle = platform_state.0.active_window_data.lock();
        *handle = Some(ActiveWindowData {
            inner_x: inner.x,
            inner_y: inner.y,
            outer_x: outer.x,
            outer_y: outer.y,
            scale: hook.scale,
        });
    } else {
        proxy.send_event(Event::WindowEvent {
            window_id: AUTOCOMPLETE_ID,
            window_event: WindowEvent::Hide,
        })?;
    }

    Ok(())
}
