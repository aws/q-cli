use cfg_if::cfg_if;
use fig_install::InstallComponents;
use fig_util::manifest::manifest;
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
use crate::platform::PlatformState;
use crate::{
    DebugState,
    EventLoopProxy,
    EventLoopWindowTarget,
    AUTOCOMPLETE_ID,
    DASHBOARD_ID,
};

const COMMANDKEY: &[u8] = include_bytes!(concat!(env!("TRAY_ICONS_PROCESSED"), "/commandkey.png",));

const GEAR: &[u8] = include_bytes!(concat!(env!("TRAY_ICONS_PROCESSED"), "/gear.png",));

const QUESTION: &[u8] = include_bytes!(concat!(env!("TRAY_ICONS_PROCESSED"), "/question.png",));

const DISCORD: &[u8] = include_bytes!(concat!(env!("TRAY_ICONS_PROCESSED"), "/discord.png",));

const GITHUB: &[u8] = include_bytes!(concat!(env!("TRAY_ICONS_PROCESSED"), "/github.png",));

pub fn handle_event(id: MenuId, proxy: &EventLoopProxy) {
    match id {
        id if id == MenuId::new("debugger-refresh") => {
            proxy.send_event(Event::ReloadTray).unwrap();
        },
        id if id == MenuId::new("dashboard-devtools") => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID,
                    window_event: WindowEvent::Devtools,
                })
                .unwrap();
        },
        id if id == MenuId::new("autocomplete-devtools") => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: AUTOCOMPLETE_ID,
                    window_event: WindowEvent::Devtools,
                })
                .unwrap();
        },
        id if id == MenuId::new("update") => {
            tokio::runtime::Handle::current().spawn(fig_install::update(true, None));
        },
        id if id == MenuId::new("quit") => {
            proxy.send_event(Event::ControlFlow(ControlFlow::Exit)).unwrap();
        },
        id if id == MenuId::new("dashboard") => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID,
                    window_event: WindowEvent::NavigateRelative { path: "".to_owned() },
                })
                .unwrap();

            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID,
                    window_event: WindowEvent::Show,
                })
                .unwrap();
        },
        id if id == MenuId::new("show") => {
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
        id if id == MenuId::new("uninstall") => {
            tokio::runtime::Handle::current().spawn(async {
                fig_install::uninstall(InstallComponents::all()).await.ok();
                std::process::exit(0);
            });
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
                .args(["issue", "--force", "bug: "])
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
    Info(String),
    Entry {
        emoji_icon: Option<String>,
        image_icon: Option<wry::application::window::Icon>,
        text: String,
        id: String,
    },
    Separator,
    SubMenu {
        title: String,
        elements: Vec<MenuElement>,
    },
}

impl MenuElement {
    fn entry(
        emoji_icon: Option<String>,
        image: Option<&'static [u8]>,
        text: impl Into<String>,
        id: impl Into<String>,
    ) -> Self {
        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                let image_icon = match image {
                    Some(image) => {
                        let image = image::load_from_memory(image)
                            .expect("Failed to open icon path")
                            .to_rgba8();

                        let (width, height) = image.dimensions();

                        Icon::from_rgba(image.into_raw(), width, height).ok()
                    },
                    None => None,
                };
            } else {
                let image_icon = None;
            }
        };

        Self::Entry {
            emoji_icon,
            image_icon,
            text: text.into(),
            id: id.into(),
        }
    }

    fn sub_menu(title: impl Into<String>, elements: Vec<MenuElement>) -> Self {
        Self::SubMenu {
            title: title.into(),
            elements,
        }
    }

    fn add_to_menu(&self, menu: &mut ContextMenu) {
        match self {
            MenuElement::Info(info) => {
                menu.add_item(MenuItemAttributes::new(info).with_enabled(false));
            },
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
                    custom_menu_item.set_icon(image_icon.clone());
                }
            },
            MenuElement::Separator => {
                menu.add_native_item(MenuItem::Separator);
            },
            MenuElement::SubMenu { title, elements } => {
                let mut sub_menu = ContextMenu::new();
                for element in elements {
                    element.add_to_menu(&mut sub_menu);
                }

                menu.add_submenu(title, true, sub_menu);
            },
        }
    }
}

fn menu() -> Vec<MenuElement> {
    let logged_in = fig_request::auth::is_logged_in();

    let report = MenuElement::entry(Some("🐞".to_owned()), Some(GITHUB), "Report an Issue", "issue");
    let manual = MenuElement::entry(Some("📚".to_owned()), Some(QUESTION), "User Manual", "user-manual");
    let discord = MenuElement::entry(Some("💬".to_owned()), Some(DISCORD), "Join Community", "community");
    let version = MenuElement::Info(format!(
        "Version {} {}",
        env!("CARGO_PKG_VERSION"),
        manifest()
            .as_ref()
            .map(|m| m.default_channel.to_string())
            .unwrap_or_default()
    ));
    let update = MenuElement::entry(None, None, "Check for updates...", "update");
    let quit = MenuElement::entry(None, None, "Quit Fig", "quit");
    let dashboard = MenuElement::entry(Some("🎛️".to_owned()), Some(COMMANDKEY), "Dashboard", "dashboard");
    let settings = MenuElement::entry(Some("⚙️".to_owned()), Some(GEAR), "Settings", "settings");
    let developer = MenuElement::sub_menu("Developer", vec![
        MenuElement::entry(None, None, "Dashboard Devtools", "dashboard-devtools"),
        MenuElement::entry(None, None, "Autocomplete Devtools", "autocomplete-devtools"),
    ]);

    if !logged_in {
        vec![
            MenuElement::Info("Fig hasn't been set up yet...".to_owned()),
            MenuElement::entry(None, None, "Get Started", "show"),
            MenuElement::Separator,
            report,
            manual,
            discord,
            MenuElement::Separator,
            MenuElement::entry(None, None, "Uninstall Fig", "uninstall"),
            MenuElement::Separator,
            quit,
        ]
    } else if !PlatformState::accessibility_is_enabled().unwrap_or(true) {
        vec![
            MenuElement::Info("Accessibility isn't enabled".to_owned()),
            MenuElement::entry(None, None, "Enable Accessibility", "accessibility"),
            MenuElement::Separator,
            dashboard,
            settings,
            MenuElement::Separator,
            manual,
            discord,
            MenuElement::Separator,
            report,
            MenuElement::Separator,
            version,
            update,
            MenuElement::Separator,
            developer,
            MenuElement::Separator,
            quit,
        ]
    } else {
        vec![
            dashboard,
            settings,
            MenuElement::Separator,
            manual,
            discord,
            MenuElement::Separator,
            report,
            MenuElement::Separator,
            version,
            update,
            MenuElement::Separator,
            developer,
            MenuElement::Separator,
            quit,
        ]
    }
}
