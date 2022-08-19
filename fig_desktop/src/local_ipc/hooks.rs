use anyhow::Result;
use fig_proto::local::{
    CursorPositionHook,
    FileChangedHook,
    FocusChangeHook,
    FocusedWindowDataHook,
};
use tracing::debug;

use crate::event::WindowEvent;
use crate::native::NativeState;
use crate::{
    Event,
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

pub async fn caret_position(
    CursorPositionHook { x, y, width, height }: CursorPositionHook,
    _proxy: &EventLoopProxy,
) -> Result<()> {
    debug!({ x, y, width, height }, "Cursor Position (ignored!)");

    Ok(())
}

pub async fn focus_change(_: FocusChangeHook, proxy: &EventLoopProxy) -> Result<()> {
    proxy
        .send_event(Event::WindowEvent {
            window_id: AUTOCOMPLETE_ID.clone(),
            window_event: WindowEvent::Hide,
        })
        .unwrap();

    Ok(())
}

pub async fn file_changed(_file_changed_hook: FileChangedHook) -> Result<()> {
    Ok(())
}

pub async fn focused_window_data(
    hook: FocusedWindowDataHook,
    native_state: &NativeState,
    proxy: &EventLoopProxy,
) -> Result<()> {
    #[cfg(target_os = "linux")]
    return crate::native::integrations::from_hook(hook, native_state, proxy);
    #[cfg(not(target_os = "linux"))]
    {
        let _hook = hook;
        let _native_state = native_state;
        let _proxy = proxy;
        Ok(())
    }
}
