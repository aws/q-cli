use fig_desktop_api::requests::{
    RequestResult,
    RequestResultImpl,
};
use fig_proto::fig::UserLogoutRequest;

use crate::event::{
    Event,
    WindowEvent,
};
use crate::webview::LOGIN_PATH;
use crate::{
    EventLoopProxy,
    DASHBOARD_ID,
};

pub async fn logout(_request: UserLogoutRequest, proxy: &EventLoopProxy) -> RequestResult {
    auth::logout().await.ok();

    proxy
        .send_event(Event::WindowEvent {
            window_id: DASHBOARD_ID,
            window_event: WindowEvent::Batch(vec![
                WindowEvent::NavigateRelative {
                    path: LOGIN_PATH.into(),
                },
                WindowEvent::Show,
            ]),
        })
        .ok();

    RequestResult::success()
}
