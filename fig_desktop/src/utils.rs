use http::header::CONTENT_TYPE;
use http::status::StatusCode;
use http::{
    Request as HttpRequest,
    Response as HttpResponse,
};
use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use wry::application::dpi::{
    Position,
    Size,
};
use wry::application::window::Icon;

/// Determines if the build is ran in debug mode
pub fn is_cargo_debug_build() -> bool {
    cfg!(debug_assertions) && !fig_settings::state::get_bool_or("developer.override-cargo-debug", false)
}

pub fn wrap_custom_protocol(
    f: impl Fn(&HttpRequest<Vec<u8>>) -> anyhow::Result<HttpResponse<Vec<u8>>> + 'static,
) -> impl Fn(&HttpRequest<Vec<u8>>) -> wry::Result<HttpResponse<Vec<u8>>> + 'static {
    move |req: &HttpRequest<Vec<u8>>| -> wry::Result<HttpResponse<Vec<u8>>> {
        Ok(match f(req) {
            Ok(res) => res,
            Err(err) => {
                let response = HttpResponse::builder().status(StatusCode::BAD_REQUEST);
                match req
                    .headers()
                    .get("Accept")
                    .and_then(|accept| accept.to_str().ok())
                    .and_then(|accept| accept.split('/').last())
                {
                    Some("json") => response.header(CONTENT_TYPE, "application/json").body(
                        serde_json::to_vec(&json!({ "error": err.to_string() })).unwrap_or_else(|_| b"{}".to_vec()),
                    ),
                    _ => response
                        .header(CONTENT_TYPE, "text/plain")
                        .body(err.to_string().into_bytes()),
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
        let image = image::load_from_memory(include_bytes!("../icons/512x512.png"))
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
