use std::process::exit;

use fig_proto::local::command_response::Response as CommandResponseTypes;
use fig_proto::local::dump_state_command::Type as DumpStateType;
use fig_proto::local::{
    DebugModeCommand,
    DiagnosticsCommand,
    DiagnosticsResponse,
    DumpStateCommand,
    DumpStateResponse,
    LogLevelCommand,
    LogLevelResponse,
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
use crate::figterm::FigtermState;
use crate::webview::DASHBOARD_ONBOARDING_SIZE;
use crate::{
    platform,
    EventLoopProxy,
    AUTOCOMPLETE_ID,
    DASHBOARD_ID,
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

pub async fn diagnostic(_: DiagnosticsCommand, figterm_state: &FigtermState) -> LocalResult {
    let (edit_buffer_string, edit_buffer_cursor, shell_context) = {
        match figterm_state.most_recent() {
            Some(session) => (
                Some(session.edit_buffer.text.clone()),
                Some(session.edit_buffer.cursor),
                session.context.clone(),
            ),
            None => (None, None, None),
        }
    };

    let response = DiagnosticsResponse {
        autocomplete_active: Some(platform::autocomplete_active()),
        #[cfg(target_os = "macos")]
        path_to_bundle: macos_accessibility_position::bundle::get_bundle_path()
            .and_then(|path| path.to_str().map(|s| s.to_owned()))
            .unwrap_or_default(),
        #[cfg(target_os = "macos")]
        accessibility: if macos_accessibility_position::accessibility::accessibility_is_enabled() {
            "true".into()
        } else {
            "false".into()
        },

        edit_buffer_string,
        edit_buffer_cursor,
        shell_context,

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
                    window_id: DASHBOARD_ID.clone(),
                    window_event: WindowEvent::NavigateRelative {
                        path: "/settings".into(),
                    },
                })
                .unwrap();
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID.clone(),
                    window_event: WindowEvent::Show,
                })
                .unwrap();
        },
        UiElement::MissionControl => {
            if let Some(path) = command.route {
                proxy
                    .send_event(Event::WindowEvent {
                        window_id: DASHBOARD_ID.clone(),
                        window_event: WindowEvent::NavigateRelative { path },
                    })
                    .unwrap();
            }

            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID.clone(),
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
    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            use fig_desktop_api::requests::install::install;
            use fig_proto::fig::{InstallRequest, InstallComponent, InstallAction};

            install(InstallRequest {
                component: InstallComponent::Accessibility.into(),
                action: InstallAction::InstallAction.into()
            }).await.ok();
            Ok(LocalResponse::Success(None))
        } else {
            Err(LocalResponse::Error {
                code: None,
                message: Some("Accessibility API not supported on this platform".to_owned()),
            })
        }
    }
}

pub fn log_level(LogLevelCommand { level }: LogLevelCommand) -> LocalResult {
    let old_level = fig_log::set_fig_log_level(level).map_err(|err| LocalResponse::Error {
        code: None,
        message: Some(format!("Error setting log level: {err}")),
    })?;

    Ok(LocalResponse::Message(Box::new(CommandResponseTypes::LogLevel(
        LogLevelResponse {
            old_level: Some(old_level),
        },
    ))))
}

pub async fn logout(proxy: &EventLoopProxy) -> LocalResult {
    proxy
        .send_event(Event::WindowEvent {
            window_id: DASHBOARD_ID,
            window_event: WindowEvent::NavigateRelative {
                path: "/onboarding/welcome".to_owned(),
            },
        })
        .ok();

    proxy
        .send_event(Event::WindowEvent {
            window_id: DASHBOARD_ID,
            window_event: WindowEvent::Resize {
                size: DASHBOARD_ONBOARDING_SIZE,
            },
        })
        .ok();

    proxy
        .send_event(Event::WindowEvent {
            window_id: DASHBOARD_ID,
            window_event: WindowEvent::Center,
        })
        .ok();

    Ok(LocalResponse::Success(None))
}

pub fn dump_state(command: DumpStateCommand, figterm_state: &FigtermState) -> LocalResult {
    let json = match command.r#type() {
        DumpStateType::DumpStateFigterm => {
            serde_json::to_string_pretty(&figterm_state).unwrap_or_else(|err| format!("unable to dump: {err}"))
        },
    };

    LocalResult::Ok(LocalResponse::Message(Box::new(CommandResponseTypes::DumpState(
        DumpStateResponse { json },
    ))))
}
