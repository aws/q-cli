use fig_proto::local::{
    DebugModeCommand,
    OpenUiElementCommand,
    UiElement,
};
use wry::application::event_loop::EventLoopProxy;

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

pub async fn open_ui_element(command: OpenUiElementCommand, proxy: &EventLoopProxy<FigEvent>) -> LocalResult {
    match command.element() {
        UiElement::MenuBar => todo!(),
        UiElement::Settings => todo!(),
        UiElement::MissionControl => proxy
            .send_event(FigEvent::WindowEvent {
                fig_id: MISSION_CONTROL_ID.clone(),
                window_event: FigWindowEvent::Show,
            })
            .unwrap(),
        UiElement::InputMethodPrompt => todo!(),
    };

    Ok(LocalResponse::Success(None))
}
