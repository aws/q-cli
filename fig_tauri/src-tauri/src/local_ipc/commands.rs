use std::process::exit;

use fig_proto::local::command_response::Response as CommandResponseTypes;
use fig_proto::local::{
    DebugModeCommand,
    DiagnosticsCommand,
    DiagnosticsResponse,
    OpenUiElementCommand,
    QuitCommand,
    UiElement,
};
use tracing::error;
use wry::application::event_loop::{
    ControlFlow,
    EventLoopProxy,
};

use super::{
    LocalResponse,
    LocalResult,
};
use crate::window::FigWindowEvent;
use crate::{
    FigEvent,
    MISSION_CONTROL_ID,
};

pub async fn debug(_command: DebugModeCommand) -> LocalResult {
    todo!()
}

pub async fn quit(_command: QuitCommand, proxy: &EventLoopProxy<FigEvent>) -> LocalResult {
    proxy
        .send_event(FigEvent::ControlFlow(ControlFlow::Exit))
        .map(|_| LocalResponse::Success(None))
        .map_err(|_| exit(0))
}

pub async fn diagnostic(_command: DiagnosticsCommand) -> LocalResult {
    let response = DiagnosticsResponse { ..Default::default() };
    Ok(LocalResponse::Message(Box::new(CommandResponseTypes::Diagnostics(
        response,
    ))))
}

pub async fn open_ui_element(command: OpenUiElementCommand, proxy: &EventLoopProxy<FigEvent>) -> LocalResult {
    match command.element() {
        UiElement::Settings => {
            proxy
                .send_event(FigEvent::WindowEvent {
                    fig_id: MISSION_CONTROL_ID.clone(),
                    window_event: FigWindowEvent::Navigate {
                        url: url::Url::parse("https://desktop.fig.io/settings/general/application").unwrap(),
                    },
                })
                .unwrap();
            proxy
                .send_event(FigEvent::WindowEvent {
                    fig_id: MISSION_CONTROL_ID.clone(),
                    window_event: FigWindowEvent::Show,
                })
                .unwrap();
        },
        UiElement::MissionControl => proxy
            .send_event(FigEvent::WindowEvent {
                fig_id: MISSION_CONTROL_ID.clone(),
                window_event: FigWindowEvent::Show,
            })
            .unwrap(),
        UiElement::MenuBar => error!("Opening menu bar is unimplemented"),
        UiElement::InputMethodPrompt => error!("Opening input method prompt is unimplemented"),
    };

    Ok(LocalResponse::Success(None))
}
