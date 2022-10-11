use once_cell::sync::Lazy;
use serde_json::json;
use wry::application::window::Icon;
use wry::http::status::StatusCode;
use wry::http::{
    Request as HttpRequest,
    Response as HttpResponse,
    ResponseBuilder as HttpResponseBuilder,
};

pub fn wrap_custom_protocol(
    f: impl Fn(&HttpRequest) -> anyhow::Result<HttpResponse>,
) -> impl Fn(&HttpRequest) -> wry::Result<HttpResponse> {
    move |req: &HttpRequest| -> wry::Result<HttpResponse> {
        Ok(match f(req) {
            Ok(res) => res,
            Err(err) => {
                let response = HttpResponseBuilder::new().status(StatusCode::BAD_REQUEST);
                match req
                    .headers()
                    .get("Accept")
                    .and_then(|accept| accept.to_str().ok())
                    .and_then(|accept| accept.split('/').last())
                {
                    Some("json") => response.mimetype("application/json").body(
                        serde_json::to_vec(&json!({ "error": err.to_string() })).unwrap_or_else(|_| b"{}".to_vec()),
                    ),
                    _ => response.mimetype("text/plain").body(err.to_string().into_bytes()),
                }?
            },
        })
    }
}

#[allow(clippy::needless_return)]
pub static ICON: Lazy<Icon> = Lazy::new(|| {
    cfg_if::cfg_if!(
        if #[cfg(target_os = "linux")] {
            return load_icon(
                fig_util::search_xdg_data_dirs("icons/hicolor/512x512/apps/fig.png")
                    .unwrap_or_else(|| "/usr/share/icons/hicolor/512x512/apps/fig.png".into()),
            );
        } else {
            return load_from_memory();
        }
    );
});

#[cfg(target_os = "linux")]
fn load_icon(path: impl AsRef<std::path::Path>) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path).expect("Failed to open icon path").into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

#[cfg(not(target_os = "linux"))]
fn load_from_memory() -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        // TODO: Use different per platform icons
        let image = image::load_from_memory(include_bytes!("../../icons/512x512.png"))
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
