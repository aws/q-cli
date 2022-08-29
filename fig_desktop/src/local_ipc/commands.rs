use std::process::exit;

use fig_proto::local::command_response::Response as CommandResponseTypes;
use fig_proto::local::{
    DebugModeCommand,
    DiagnosticsCommand,
    DiagnosticsResponse,
    OpenUiElementCommand,
    QuitCommand,
    UiElement,
    UpdateCommand,
};
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
    native,
    EventLoopProxy,
    MISSION_CONTROL_ID,
};

pub async fn debug(_: DebugModeCommand) -> LocalResult {
    todo!()
}

pub async fn quit(_: QuitCommand, proxy: &EventLoopProxy) -> LocalResult {
    proxy
        .send_event(Event::ControlFlow(ControlFlow::Exit))
        .map(|_| LocalResponse::Success(None))
        .map_err(|_| exit(0))
}

pub async fn diagnostic(_: DiagnosticsCommand) -> LocalResult {
    let response = DiagnosticsResponse {
        autocomplete_active: Some(native::autocomplete_active()),
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
                    window_event: WindowEvent::Navigate {
                        url: url::Url::parse("https://desktop.fig.io/settings/general/application").unwrap(),
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
            if let Some(route) = command.route {
                let mut url = url::Url::parse("https://desktop.fig.io").unwrap();
                url.set_path(&route);

                proxy
                    .send_event(Event::WindowEvent {
                        window_id: MISSION_CONTROL_ID.clone(),
                        window_event: WindowEvent::Navigate { url },
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

pub async fn update(_command: UpdateCommand) -> LocalResult {
    #[cfg(windows)]
    crate::utils::update_check().await;
    Ok(LocalResponse::Success(None))
}
