use std::borrow::Cow;

use cfg_if::cfg_if;
use fig_install::{
    InstallComponents,
    UpdateOptions,
};
use fig_integrations::shell::ShellExt;
use fig_util::directories::relative_cli_path;
use fig_util::manifest::{
    manifest,
    Channel,
};
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
use crate::webview::ONBOARDING_PATH;
use crate::{
    DebugState,
    EventLoopProxy,
    EventLoopWindowTarget,
    AUTOCOMPLETE_ID,
    DASHBOARD_ID,
};

macro_rules! icon {
    ($icon:literal) => {{
        #[cfg(target_os = "macos")]
        {
            Some(include_bytes!(concat!(
                env!("TRAY_ICONS_PROCESSED"),
                "/",
                $icon,
                ".png"
            )))
        }
        #[cfg(not(target_os = "macos"))]
        {
            None
        }
    }};
}

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
            let proxy_a = proxy.clone();
            let proxy_b = proxy.clone();
            tokio::runtime::Handle::current().spawn(async move {
                match fig_install::update(
                    Some(Box::new(move |_| {
                        proxy_a
                            .send_event(Event::ShowMessageNotification {
                                title: "Fig Update".into(),
                                body:
                                    "Fig is updating in the background. You can continue to use Fig while it updates."
                                        .into(),
                                parent: None,
                            })
                            .unwrap();
                    })),
                    UpdateOptions {
                        ignore_rollout: true,
                        interactive: true,
                        relaunch_dashboard: true,
                    },
                )
                .await
                {
                    Ok(true) => {},
                    Ok(false) => {
                        // Didn't update, show a notification
                        proxy_b
                            .send_event(Event::ShowMessageNotification {
                                title: "Fig Update".into(),
                                body: concat!("Fig is already up to date. Version (", env!("CARGO_PKG_VERSION"), ")")
                                    .into(),
                                parent: None,
                            })
                            .unwrap();
                    },
                    Err(err) => {
                        // Error updating, show a notification
                        proxy_b
                            .send_event(Event::ShowMessageNotification {
                                title: "Fig Update".into(),
                                body: format!("Error updating Fig: {err}").into(),
                                parent: None,
                            })
                            .unwrap();
                    },
                }
            });
        },
        id if id == MenuId::new("quit") => {
            proxy.send_event(Event::ControlFlow(ControlFlow::Exit)).unwrap();
        },
        id if id == MenuId::new("dashboard") => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID.clone(),
                    window_event: WindowEvent::Batch(vec![
                        WindowEvent::NavigateRelative { path: "/".into() },
                        WindowEvent::Show,
                    ]),
                })
                .unwrap();
        },
        id if id == MenuId::new("onboarding") => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID.clone(),
                    window_event: WindowEvent::Batch(vec![
                        WindowEvent::NavigateRelative {
                            path: ONBOARDING_PATH.into(),
                        },
                        WindowEvent::Show,
                    ]),
                })
                .unwrap();
        },
        id if id == MenuId::new("settings") => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID.clone(),
                    window_event: WindowEvent::Batch(vec![
                        WindowEvent::NavigateRelative {
                            path: "/settings".into(),
                        },
                        WindowEvent::Show,
                    ]),
                })
                .unwrap();
        },
        id if id == MenuId::new("not-working") => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID.clone(),
                    window_event: WindowEvent::Batch(vec![
                        WindowEvent::NavigateRelative {
                            path: "?show_help=true".into(),
                        },
                        WindowEvent::Show,
                    ]),
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
        id if id == MenuId::new("issue") => match relative_cli_path() {
            Ok(fig_cli) => {
                std::process::Command::new(fig_cli)
                    .args(["issue", "--force", "bug: "])
                    .output()
                    .ok();
            },
            Err(err) => error!(%err, "Failed to execute `fig issue` from the tray"),
        },
        id => {
            trace!(?id, "Unhandled tray event");
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
    Info(Cow<'static, str>),
    Entry {
        emoji_icon: Option<Cow<'static, str>>,
        image_icon: Option<wry::application::window::Icon>,
        text: Cow<'static, str>,
        id: Cow<'static, str>,
    },
    Separator,
    SubMenu {
        title: Cow<'static, str>,
        elements: Vec<MenuElement>,
    },
}

impl MenuElement {
    fn entry(
        emoji_icon: Option<Cow<'static, str>>,
        image: Option<&'static [u8]>,
        text: impl Into<Cow<'static, str>>,
        id: impl Into<Cow<'static, str>>,
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
                let _ = image;
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

    fn sub_menu(title: impl Into<Cow<'static, str>>, elements: Vec<MenuElement>) -> Self {
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
                    _ => text.to_string(),
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

    let not_working = MenuElement::entry(Some("🚨".into()), icon!("alert"), "Fig isn't working?", "not-working");
    let report = MenuElement::entry(Some("🐞".into()), icon!("github"), "Report an Issue", "issue");
    let manual = MenuElement::entry(Some("📚".into()), icon!("question"), "User Manual", "user-manual");
    let discord = MenuElement::entry(Some("💬".into()), icon!("discord"), "Join Community", "community");
    let version = MenuElement::Info(format!("Version: {}", env!("CARGO_PKG_VERSION")).into());
    let update = MenuElement::entry(None, None, "Check for updates...", "update");
    let quit = MenuElement::entry(None, None, "Quit Fig", "quit");
    let dashboard = MenuElement::entry(Some("🎛️".into()), icon!("commandkey"), "Dashboard", "dashboard");
    let settings = MenuElement::entry(Some("⚙️".into()), icon!("gear"), "Settings", "settings");
    let developer = MenuElement::sub_menu("Developer", vec![
        MenuElement::entry(None, None, "Dashboard Devtools", "dashboard-devtools"),
        MenuElement::entry(None, None, "Autocomplete Devtools", "autocomplete-devtools"),
    ]);

    let mut menu = if !logged_in {
        vec![
            MenuElement::Info("Fig hasn't been set up yet...".into()),
            MenuElement::entry(None, None, "Get Started", "onboarding"),
            MenuElement::Separator,
            manual,
            discord,
            MenuElement::Separator,
            not_working,
            report,
            MenuElement::Separator,
            MenuElement::entry(None, None, "Uninstall Fig", "uninstall"),
        ]
    } else {
        let mut menu = vec![];

        // accessibility not enabled
        // or shell integrations are not installed,
        // or input method not enabled AND kitty/alacritty/jetbrains installed

        let handle = tokio::runtime::Handle::current();
        let shell_not_installed = std::thread::spawn(move || {
            fig_util::Shell::all()
                .iter()
                .filter_map(|s| s.get_shell_integrations().ok())
                .flatten()
                .any(|i| handle.block_on(i.is_installed()).is_err())
        })
        .join()
        .unwrap();

        let accessibility_not_installed = !PlatformState::accessibility_is_enabled().unwrap_or(true);

        // TODO: Add input method check

        if accessibility_not_installed || shell_not_installed {
            menu.extend([
                MenuElement::Info("Fig hasn't been configured correctly".into()),
                MenuElement::entry(None, None, "Fix Configuration Issues", "not-working"),
                MenuElement::Separator,
            ]);
        }

        menu.extend([
            dashboard,
            settings,
            MenuElement::Separator,
            manual,
            discord,
            MenuElement::Separator,
            not_working,
            report,
            MenuElement::Separator,
            developer,
        ]);

        menu
    };

    menu.extend([MenuElement::Separator, version]);

    if let Some(channel) = manifest().as_ref().map(|m| m.default_channel) {
        if channel != Channel::Stable {
            menu.push(MenuElement::Info(format!("Channel: {channel}").into()));
        }
    }

    menu.extend([update, MenuElement::Separator, quit]);

    menu
}
