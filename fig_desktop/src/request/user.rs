use fig_desktop_api::requests::{
    RequestResult,
    RequestResultImpl,
};
use fig_proto::fig::UserLogoutRequest;

use crate::event::{
    Event,
    WindowEvent,
};
use crate::webview::{
    DASHBOARD_ONBOARDING_SIZE,
    ONBOARDING_PATH,
};
use crate::{
    EventLoopProxy,
    DASHBOARD_ID,
};

pub fn logout(_request: UserLogoutRequest, proxy: &EventLoopProxy) -> RequestResult {
    fig_request::auth::logout().ok();

    proxy
        .send_event(Event::WindowEvent {
            window_id: DASHBOARD_ID,
            window_event: WindowEvent::NavigateRelative {
                path: ONBOARDING_PATH.into(),
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

    RequestResult::success()
}
