use once_cell::sync::Lazy;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use wry::application::dpi::{
    LogicalPosition,
    LogicalSize,
};
use wry::application::window::Icon;
use wry::http::status::StatusCode;
use wry::http::{
    Request as HttpRequest,
    Response as HttpResponse,
    ResponseBuilder as HttpResponseBuilder,
};

/// Determines if the build is ran in debug mode
pub fn is_cargo_debug_build() -> bool {
    cfg!(debug_assertions) && !fig_settings::state::get_bool_or("developer.override-cargo-debug", false)
}

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
    pub position: LogicalPosition<f64>,
    pub size: LogicalSize<f64>,
}

impl Rect {
    pub fn left(&self) -> f64 {
        self.position.x
    }

    pub fn right(&self) -> f64 {
        self.position.x + self.size.width
    }

    pub fn center(&self) -> f64 {
        self.position.x + self.size.width * 0.5
    }

    pub fn top(&self) -> f64 {
        self.position.y
    }

    pub fn bottom(&self) -> f64 {
        self.position.y + self.size.height
    }

    pub fn middle(&self) -> f64 {
        self.position.y + self.size.height * 0.5
    }

    pub fn contains(&self, point: LogicalPosition<f64>) -> bool {
        point.x >= self.position.x
            && point.x <= self.position.x + self.size.width
            && point.y >= self.position.y
            && point.y <= self.position.y + self.size.height
    }
}
