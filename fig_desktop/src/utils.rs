use std::borrow::Cow;
use std::future::Future;
use std::sync::atomic::{
    AtomicU64,
    Ordering,
};

use http::header::{
    ACCESS_CONTROL_ALLOW_ORIGIN,
    CONTENT_TYPE,
};
use http::status::StatusCode;
use http::{
    HeaderValue,
    Request as HttpRequest,
    Response as HttpResponse,
};
use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use tao::dpi::{
    Position,
    Size,
};
use tao::window::Icon;
use tracing::{
    debug,
    debug_span,
    error,
    Instrument,
};
use wry::RequestAsyncResponder;

/// Determines if the build is ran in debug mode
pub fn is_cargo_debug_build() -> bool {
    cfg!(debug_assertions) && !fig_settings::state::get_bool_or("developer.override-cargo-debug", false)
}

pub fn wrap_custom_protocol<F, Fut>(
    proto_name: &'static str,
    f: F,
) -> impl Fn(HttpRequest<Vec<u8>>, RequestAsyncResponder) + 'static
where
    F: Fn(HttpRequest<Vec<u8>>) -> Fut + Send + Copy + 'static,
    Fut: Future<Output = anyhow::Result<HttpResponse<Cow<'static, [u8]>>>> + Send + 'static,
{
    move |req: HttpRequest<Vec<u8>>, responder: RequestAsyncResponder| {
        let proto = proto_name;

        static ID_CTR: AtomicU64 = AtomicU64::new(0);
        let id = ID_CTR.fetch_add(1, Ordering::Relaxed);

        let span = debug_span!("custom-proto", %proto, %id);
        let _enter = span.enter();

        tokio::spawn(
            async move {
                debug!(uri =% req.uri(), "{proto_name} proto request");

                let accept_json = req
                    .headers()
                    .get("Accept")
                    .and_then(|accept| accept.to_str().ok())
                    .and_then(|accept| accept.split('/').last())
                    .map_or(false, |accept| accept == "json");

                let mut response = match f(req).await {
                    Ok(res) => res,
                    Err(err) => {
                        error!(%err, "Custom protocol failed");

                        let response = HttpResponse::builder()
                            .status(StatusCode::INTERNAL_SERVER_ERROR)
                            .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");
                        if accept_json {
                            response.header(CONTENT_TYPE, "application/json").body(
                                serde_json::to_vec(&json!({ "error": err.to_string() }))
                                    .map_or_else(|_| b"{}".as_ref().into(), Into::into),
                            )
                        } else {
                            response
                                .header(CONTENT_TYPE, "text/plain")
                                .body(err.to_string().into_bytes().into())
                        }
                        .unwrap()
                    },
                };

                response
                    .headers_mut()
                    .insert("X-Request-Id", HeaderValue::from_str(&id.to_string()).unwrap());

                debug!(status = %response.status(), "Custom proto response");

                responder.respond(response);
            }
            .in_current_span(),
        );
    }
}

#[allow(clippy::needless_return)]
pub static ICON: Lazy<Icon> = Lazy::new(|| {
    cfg_if::cfg_if!(
        if #[cfg(target_os = "linux")] {
            return load_icon(
                fig_util::search_xdg_data_dirs("icons/hicolor/512x512/apps/fig.png")
                    .unwrap_or_else(|| "/usr/share/icons/hicolor/512x512/apps/fig.png".into()),
            ).unwrap_or_else(load_from_memory);
        } else {
            return load_from_memory();
        }
    );
});

#[cfg(target_os = "linux")]
fn load_icon(path: impl AsRef<std::path::Path>) -> Option<Icon> {
    let image = image::open(path).ok()?.into_rgba8();
    let (width, height) = image.dimensions();
    let rgba = image.into_raw();
    Icon::from_rgba(rgba, width, height).ok()
}

fn load_from_memory() -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        // TODO: Use different per platform icons
        let image = image::load_from_memory(include_bytes!("../icons/icon.png"))
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(target_os = "linux", ignore)]
    #[test]
    fn icon() {
        let _icon = &*ICON;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
// todo: rename to LogicalFrame
// A logical rect, where the origin point is the top left corner.
pub struct Rect {
    pub position: Position,
    pub size: Size,
}

#[allow(dead_code)]
impl Rect {
    pub fn left(&self, scale_factor: f64) -> f64 {
        self.position.to_logical::<f64>(scale_factor).x
    }

    pub fn right(&self, scale_factor: f64) -> f64 {
        self.position.to_logical::<f64>(scale_factor).x + self.size.to_logical::<f64>(scale_factor).width
    }

    pub fn top(&self, scale_factor: f64) -> f64 {
        self.position.to_logical::<f64>(scale_factor).y
    }

    pub fn bottom(&self, scale_factor: f64) -> f64 {
        self.position.to_logical::<f64>(scale_factor).y + self.size.to_logical::<f64>(scale_factor).height
    }

    pub fn contains(&self, point: Position, scale_factor: f64) -> bool {
        let point = point.to_logical::<f64>(scale_factor);

        let rect_position = self.position.to_logical::<f64>(scale_factor);
        let rect_size = self.size.to_logical::<f64>(scale_factor);

        let contains_x = point.x >= rect_position.x && point.x <= rect_position.x + rect_size.width;
        let contains_y = point.y >= rect_position.y && point.y <= rect_position.y + rect_size.height;

        contains_x && contains_y
    }
}
