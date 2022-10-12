use cfg_if::cfg_if;
use tracing::{
    error,
    trace,
};
use wry::application::event_loop::ControlFlow;
use wry::application::menu::{
    ContextMenu,
    MenuId,
    MenuItem,
    MenuItemAttributes,
};
#[cfg(target_os = "macos")]
use wry::application::platform::macos::SystemTrayBuilderExtMacOS;
use wry::application::system_tray::{
    Icon,
    SystemTray,
    SystemTrayBuilder,
};

use crate::event::{
    Event,
    WindowEvent,
};
use crate::figterm::FigtermState;
use crate::{
    DebugState,
    EventLoopProxy,
    EventLoopWindowTarget,
    AUTOCOMPLETE_ID,
    DASHBOARD_ID,
};

pub fn handle_event(id: MenuId, proxy: &EventLoopProxy) {
    match id {
        id if id == MenuId::new("debugger-refresh") => {
            proxy.send_event(Event::ReloadTray).unwrap();
        },
        id if id == MenuId::new("toggle-devtools") => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: AUTOCOMPLETE_ID,
                    window_event: WindowEvent::Devtools,
                })
                .unwrap();
        },
        id if id == MenuId::new("quit") => {
            proxy.send_event(Event::ControlFlow(ControlFlow::Exit)).unwrap();
        },
        id if id == MenuId::new("dashboard") || id == MenuId::new("accessibility") || id == MenuId::new("login") => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID,
                    window_event: WindowEvent::Show,
                })
                .unwrap();
        },
        id if id == MenuId::new("settings") => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID,
                    window_event: WindowEvent::Show,
                })
                .unwrap();
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID,
                    window_event: WindowEvent::NavigateRelative {
                        path: "/settings".into(),
                    },
                })
                .unwrap();
        },
        id if id == MenuId::new("community") => {
            if let Err(err) = fig_util::open_url("https://fig.io/community") {
                error!(%err, "Failed to open community url")
            }
        },
        id if id == MenuId::new("user-manual") => {
            if let Err(err) = fig_util::open_url("https://fig.io/user-manual") {
                error!(%err, "Failed to open user manual url")
            }
        },
        id if id == MenuId::new("issue") => {
            std::process::Command::new("fig")
                .arg("issue")
                .arg("Title")
                .output()
                .ok();
        },
        id => {
            trace!("Unhandled tray event: {id:?}");
        },
    }
}

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

#[cfg(target_os = "windows")]
fn load_from_memory() -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        // TODO: Use different per platform icons
        let image = image::load_from_memory(include_bytes!("../icons/32x32.png"))
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

#[cfg(target_os = "macos")]
fn load_from_memory() -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        // TODO: Use different per platform icons
        let image = image::load_from_memory(include_bytes!("../icons/macos-menubar-template-icon@2x-scaled.png"))
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    // TODO: account for retina display (currently image is too large!)
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

pub fn build_tray(
    event_loop_window_target: &EventLoopWindowTarget,
    _debug_state: &DebugState,
    _figterm_state: &FigtermState,
) -> wry::Result<SystemTray> {
    let tray_menu = get_context_menu();

    cfg_if!(
        if #[cfg(target_os = "linux")] {
            let icon_path = "/usr/share/icons/hicolor/64x64/apps/fig.png";
            let icon = load_icon(icon_path);
        } else {
            let icon = load_from_memory();
        }
    );

    #[allow(unused_mut)]
    let mut tray_builder = SystemTrayBuilder::new(icon, Some(tray_menu));

    #[cfg(target_os = "macos")]
    {
        tray_builder = tray_builder.with_icon_as_template(true);
    }

    Ok(tray_builder.build(event_loop_window_target)?)
}

pub fn get_context_menu() -> ContextMenu {
    let mut tray_menu = ContextMenu::new();

    let elements = menu();
    for elem in elements {
        elem.add_to_menu(&mut tray_menu);
    }

    tray_menu
}

enum MenuElement {
    Entry {
        emoji_icon: Option<String>,
        image_icon: Option<wry::application::window::Icon>,
        text: String,
        id: String,
    },
    Separator,
}

impl MenuElement {
    fn add_to_menu(&self, menu: &mut ContextMenu) {
        match self {
            MenuElement::Entry {
                emoji_icon,
                image_icon,
                text,
                id,
            } => {
                let text = match (std::env::consts::OS, emoji_icon) {
                    ("linux", Some(emoji_icon)) => format!("{} {}", emoji_icon, text),
                    _ => text.clone(),
                };
                let menu_item = MenuItemAttributes::new(&text).with_id(MenuId::new(id));
                let mut custom_menu_item = menu.add_item(menu_item);
                if let Some(image_icon) = &image_icon {
                    // TODO: account for retina display (currently image is too large!)
                    custom_menu_item.set_icon(image_icon.clone());
                }
            },
            MenuElement::Separator => {
                menu.add_native_item(MenuItem::Separator);
            },
        }
    }
}

#[cfg(target_os = "macos")]
macro_rules! load_icon {
    ($path:literal) => {{
        let (icon_rgba, icon_width, icon_height) = {
            let image = image::load_from_memory(include_bytes!(concat!(
                env!("TRAY_ICONS_PROCESSED"),
                "/",
                $path,
                ".png"
            )))
            .expect("Failed to open icon path")
            .to_rgba8();
            let (width, height) = image.dimensions();
            let rgba = image.into_raw();
            (rgba, width, height)
        };
        Icon::from_rgba(icon_rgba, icon_width, icon_height).ok()
    }};
}

macro_rules! menu_element {
    (Element, $emoji_icon:expr, $image_path:literal, $text:expr, $id:expr) => {{
        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                let image_icon = load_icon!($image_path);
            } else {
                let image_icon = None;
            }
        };

        let emoji_icon: Option<&str> = $emoji_icon;

        let entry = MenuElement::Entry {
            emoji_icon: emoji_icon.map(|emoji_icon| emoji_icon.into()),
            image_icon,
            text: $text.into(),
            id: $id.into(),
        };

        entry
    }};
    (Element, $emoji_icon:expr,None, $text:expr, $id:expr) => {{
        let entry = MenuElement::Entry {
            emoji_icon: $emoji_icon,
            image_icon: None,
            text: $text.into(),
            id: $id.into(),
        };

        entry
    }};
    (Separator) => {
        MenuElement::Separator
    };
}

fn menu() -> Vec<MenuElement> {
    let logged_in = fig_request::auth::is_logged_in();

    if !logged_in {
        return vec![
            menu_element!(Element, None, None, "Login", "login"),
            menu_element!(Separator),
            menu_element!(Element, None, None, "Quit", "quit"),
        ];
    }

    cfg_if::cfg_if! {
        if #[cfg(target_os = "macos")] {
            let has_accessibility = macos_accessibility_position::accessibility::accessibility_is_enabled();
        } else {
            let has_accessibility = true;
        }
    }

    if !has_accessibility {
        return vec![
            menu_element!(Element, None, None, "Accessibility is not enabled", "accessibility"),
            menu_element!(Separator),
            menu_element!(Element, None, None, "Quit", "quit"),
        ];
    }

    vec![
        menu_element!(Element, Some("🎛️"), "commandkey", "Dashboard", "dashboard"),
        menu_element!(Element, Some("⚙️"), "gear", "Settings", "settings"),
        menu_element!(Separator),
        menu_element!(Element, Some("📚"), "question", "User Manual", "user-manual"),
        menu_element!(Element, Some("💬"), "discord", "Join Community", "community"),
        menu_element!(Separator),
        menu_element!(Element, Some("🐞"), "github", "Report an Issue", "issue"),
        menu_element!(
            Element,
            None,
            None,
            format!("Version {}", env!("CARGO_PKG_VERSION")),
            "version"
        ),
        menu_element!(Element, None, None, "Toggle Devtools", "toggle-devtools"),
        menu_element!(Separator),
        menu_element!(Element, None, None, "Quit", "quit"),
    ]
}
