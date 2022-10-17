use anyhow::Result;
use fig_proto::local::{
    CaretPositionHook,
    FileChangedHook,
    FocusChangeHook,
    FocusedWindowDataHook,
};
use wry::application::dpi::{
    LogicalPosition,
    LogicalSize,
};

use crate::event::WindowEvent;
use crate::platform::PlatformState;
use crate::utils::Rect;
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
            window_event: WindowEvent::PositionRelativeToCaret {
                caret: Rect {
                    position: LogicalPosition::new(x, y),
                    size: LogicalSize::new(width, height),
                },
            },
        })
        .ok();

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
