use std::borrow::Cow;

use http::header::{
    ACCESS_CONTROL_ALLOW_ORIGIN,
    CONTENT_TYPE,
};
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
use tracing::error;
use url::Url;
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
    f: impl Fn(&HttpRequest<Vec<u8>>) -> anyhow::Result<HttpResponse<Cow<'static, [u8]>>> + 'static,
) -> impl Fn(&HttpRequest<Vec<u8>>) -> wry::Result<HttpResponse<Cow<'static, [u8]>>> + 'static {
    move |req: &HttpRequest<Vec<u8>>| -> wry::Result<HttpResponse<Cow<[u8]>>> {
        Ok(match f(req) {
            Ok(res) => res,
            Err(err) => {
                let response = HttpResponse::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*");
                match req
                    .headers()
                    .get("Accept")
                    .and_then(|accept| accept.to_str().ok())
                    .and_then(|accept| accept.split('/').last())
                {
                    Some("json") => response.header(CONTENT_TYPE, "application/json").body(
                        serde_json::to_vec(&json!({ "error": err.to_string() }))
                            .map(Into::into)
                            .unwrap_or_else(|_| b"{}".as_ref().into()),
                    ),
                    _ => response
                        .header(CONTENT_TYPE, "text/plain")
                        .body(err.to_string().into_bytes().into()),
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

pub fn handle_login_deep_link(url: &Url) -> Option<serde_json::Value> {
    if let Some(fragment) = url.fragment() {
        let parse = url::form_urlencoded::parse(fragment.as_bytes());
        let hash: std::collections::HashMap<_, _> = parse.collect();

        let access_token = hash.get("accessToken").map(|s| s.clone().into_owned());
        let id_token = hash.get("idToken").map(|s| s.clone().into_owned());
        let refresh_token = hash.get("refreshToken").map(|s| s.clone().into_owned());
        let email = hash.get("email").map(|s| s.clone().into_owned());

        let creds = fig_request::auth::Credentials::new_jwt(
            email.clone(),
            access_token.clone(),
            id_token.clone(),
            refresh_token.clone(),
            false,
        );

        if let Err(err) = creds.save_credentials() {
            error!(%err, "Failed to save credentials");
        }

        // Since this can happen before the webview is opened we need to persist the newUser state since all
        // other state is persisted via the credentials file, the `newUser` local state value will
        // be unset in the dashboard login page
        let new_user = if let Some(new_user) = hash.get("newUser") {
            if new_user == "true" {
                if let Err(err) = fig_settings::state::set_value("login.newUser", true) {
                    error!(%err, "Failed to set new user");
                }
                true
            } else {
                false
            }
        } else {
            false
        };

        Some(serde_json::json!({
            "accessToken": access_token,
            "idToken": id_token,
            "refreshToken": refresh_token,
            "email": email,
            "newUser": new_user
        }))
    } else {
        None
    }
}
