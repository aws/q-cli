use anyhow::Result;
use fig_proto::local::{
    CaretPositionHook,
    FileChangedHook,
    FocusedWindowDataHook,
};
use wry::application::dpi::{
    LogicalPosition,
    LogicalSize,
};

use crate::event::{
    WindowEvent,
    WindowPosition,
};
use crate::platform::PlatformState;
use crate::{
    Event,
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

pub async fn caret_position(
    CaretPositionHook { x, y, width, height }: CaretPositionHook,
    proxy: &EventLoopProxy,
) -> Result<()> {
    proxy
        .send_event(Event::WindowEvent {
            window_id: AUTOCOMPLETE_ID,
            window_event: WindowEvent::UpdateWindowGeometry {
                position: Some(WindowPosition::RelativeToCaret {
                    caret_position: LogicalPosition::new(x, y),
                    caret_size: LogicalSize::new(width, height),
                }),
                size: None,
                anchor: None,
            },
        })
        .ok();

    Ok(())
}

pub async fn focus_change(proxy: &EventLoopProxy) -> Result<()> {
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
    platform_state: &PlatformState,
    proxy: &EventLoopProxy,
) -> Result<()> {
    #[cfg(target_os = "linux")]
    return crate::platform::integrations::from_hook(hook, platform_state, proxy);
    #[cfg(not(target_os = "linux"))]
    {
        let _hook = hook;
        let _platform_state = platform_state;
        let _proxy = proxy;
        Ok(())
    }
}
