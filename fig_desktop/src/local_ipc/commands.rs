use std::process::exit;

use fig_proto::local::command_response::Response as CommandResponseTypes;
use fig_proto::local::{
    DebugModeCommand,
    DiagnosticsCommand,
    DiagnosticsResponse,
    OpenBrowserCommand,
    OpenUiElementCommand,
    QuitCommand,
    UiElement,
};
use parking_lot::Mutex;
use tracing::error;
use wry::application::event_loop::ControlFlow;

use super::{
    LocalResponse,
    LocalResult,
};
use crate::event::{
    Event,
    WindowEvent,
};
use crate::{
    platform,
    EventLoopProxy,
    AUTOCOMPLETE_ID,
    MISSION_CONTROL_ID,
};

pub async fn debug(command: DebugModeCommand, proxy: &EventLoopProxy) -> LocalResult {
    static DEBUG_MODE: Mutex<bool> = Mutex::new(false);

    let debug_mode = match command.set_debug_mode {
        Some(b) => {
            *DEBUG_MODE.lock() = b;
            b
        },
        None => match command.toggle_debug_mode {
            Some(true) => {
                let mut locked_debug = DEBUG_MODE.lock();
                *locked_debug = !*locked_debug;
                *locked_debug
            },
            _ => *DEBUG_MODE.lock(),
        },
    };

    proxy
        .send_event(Event::WindowEvent {
            window_id: AUTOCOMPLETE_ID.clone(),
            window_event: WindowEvent::DebugMode(debug_mode),
        })
        .unwrap();

    Ok(LocalResponse::Success(None))
}

pub async fn quit(_: QuitCommand, proxy: &EventLoopProxy) -> LocalResult {
    proxy
        .send_event(Event::ControlFlow(ControlFlow::Exit))
        .map(|_| LocalResponse::Success(None))
        .map_err(|_| exit(0))
}

pub async fn diagnostic(_: DiagnosticsCommand) -> LocalResult {
    let response = DiagnosticsResponse {
        autocomplete_active: Some(platform::autocomplete_active()),
        ..Default::default()
    };
    Ok(LocalResponse::Message(Box::new(CommandResponseTypes::Diagnostics(
        response,
    ))))
}

pub async fn open_ui_element(command: OpenUiElementCommand, proxy: &EventLoopProxy) -> LocalResult {
    match command.element() {
        UiElement::Settings => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: MISSION_CONTROL_ID.clone(),
                    window_event: WindowEvent::NavigateRelative {
                        path: "/settings".into(),
                    },
                })
                .unwrap();
            proxy
                .send_event(Event::WindowEvent {
                    window_id: MISSION_CONTROL_ID.clone(),
                    window_event: WindowEvent::Show,
                })
                .unwrap();
        },
        UiElement::MissionControl => {
            if let Some(path) = command.route {
                proxy
                    .send_event(Event::WindowEvent {
                        window_id: MISSION_CONTROL_ID.clone(),
                        window_event: WindowEvent::NavigateRelative { path },
                    })
                    .unwrap();
            }

            proxy
                .send_event(Event::WindowEvent {
                    window_id: MISSION_CONTROL_ID.clone(),
                    window_event: WindowEvent::Show,
                })
                .unwrap();
        },
        UiElement::MenuBar => error!("Opening menu bar is unimplemented"),
        UiElement::InputMethodPrompt => error!("Opening input method prompt is unimplemented"),
    };

    Ok(LocalResponse::Success(None))
}

pub async fn open_browser(command: OpenBrowserCommand) -> LocalResult {
    if let Err(err) = fig_util::open_url(command.url) {
        error!(%err, "Error opening browser");
    }
    Ok(LocalResponse::Success(None))
}

pub async fn prompt_for_accessibility_permission() -> LocalResult {
    cfg_if::cfg_if!(
        if #[cfg(target_os = "macos")] {
            use fig_integrations::{
                accessibility::AccessibilityIntegration,
                Integration
            };

            let integration = AccessibilityIntegration {};
            match integration.install(None) {
                Ok(()) => {
                    #[allow(clippy::needless_return)]
                    return Ok(LocalResponse::Success(None));
                }
                Err(_) => {
                    #[allow(clippy::needless_return)]
                    return Err(LocalResponse::Error {
                        code: None,
                        message: Some("Accessibility API not supported on this platform".to_owned()),
                    });
                }
            }
        } else {
            #[allow(clippy::needless_return)]
            return Err(LocalResponse::Error {
                code: None,
                message: Some("Accessibility API not supported on this platform".to_owned()),
            });
        }
    );
}
