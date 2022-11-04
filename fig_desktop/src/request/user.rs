use fig_desktop_api::requests::{
    RequestResult,
    RequestResultImpl,
};
use fig_proto::fig::UserLogoutRequest;

use crate::event::{
    Event,
    WindowEvent,
    WindowPosition,
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
            window_event: WindowEvent::Batch(vec![
                WindowEvent::NavigateRelative {
                    path: ONBOARDING_PATH.into(),
                },
                WindowEvent::UpdateWindowGeometry {
                    position: Some(WindowPosition::Centered),
                    size: Some(DASHBOARD_ONBOARDING_SIZE),
                    anchor: None,
                },
                WindowEvent::Show,
            ]),
        })
        .ok();

    RequestResult::success()
}
