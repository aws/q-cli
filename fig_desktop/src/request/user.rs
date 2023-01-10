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
    LOGIN_PATH,
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
                    path: LOGIN_PATH.into(),
                },
                WindowEvent::UpdateWindowGeometry {
                    position: Some(WindowPosition::Centered),
                    size: Some(DASHBOARD_ONBOARDING_SIZE),
                    anchor: None,
                    tx: None,
                    dry_run: false,
                },
                WindowEvent::Show,
            ]),
        })
        .ok();

    RequestResult::success()
}
