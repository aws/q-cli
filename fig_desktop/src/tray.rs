use std::borrow::Cow;

use cfg_if::cfg_if;
use fig_install::{
    InstallComponents,
    UpdateOptions,
};
use fig_util::manifest::Channel;
use muda::{
    Menu,
    MenuEvent,
    MenuId,
    MenuItem,
    MenuItemBuilder,
    PredefinedMenuItem,
    Submenu,
};
use tracing::{
    error,
    trace,
};
use tray_icon::{
    Icon,
    TrayIcon,
    TrayIconBuilder,
};
use wry::application::event_loop::ControlFlow;

use crate::event::{
    Event,
    WindowEvent,
};
use crate::figterm::FigtermState;
use crate::webview::LOGIN_PATH;
use crate::{
    DebugState,
    EventLoopProxy,
    EventLoopWindowTarget,
    AUTOCOMPLETE_ID,
    DASHBOARD_ID,
};

// macro_rules! icon {
//     ($icon:literal) => {{
//         #[cfg(target_os = "macos")]
//         {
//             Some(include_bytes!(concat!(
//                 env!("TRAY_ICONS_PROCESSED"),
//                 "/",
//                 $icon,
//                 ".png"
//             )))
//         }
//         #[cfg(not(target_os = "macos"))]
//         {
//             None
//         }
//     }};
// }

fn tray_update(proxy: &EventLoopProxy) {
    let proxy_a = proxy.clone();
    let proxy_b = proxy.clone();
    tokio::runtime::Handle::current().spawn(async move {
        match fig_install::update(
            Some(Box::new(move |_| {
                proxy_a
                    .send_event(Event::ShowMessageNotification {
                        title: "CodeWhisperer is updating in the background".into(),
                        body: "You can continue to use CodeWhisperer while it updates".into(),
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
                        title: "CodeWhisperer is already up to date".into(),
                        body: concat!("Version ", env!("CARGO_PKG_VERSION")).into(),
                        parent: None,
                    })
                    .unwrap();
            },
            Err(err) => {
                // Error updating, show a notification
                proxy_b
                    .send_event(Event::ShowMessageNotification {
                        title: "Error Updating CodeWhisperer".into(),
                        body: err.to_string().into(),
                        parent: None,
                    })
                    .unwrap();
            },
        }
    });
}

pub fn handle_event(menu_event: &MenuEvent, proxy: &EventLoopProxy) {
    match &*menu_event.id().0 {
        "debugger-refresh" => {
            proxy.send_event(Event::ReloadTray).unwrap();
        },
        "dashboard-devtools" => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID,
                    window_event: WindowEvent::Devtools,
                })
                .unwrap();
        },
        "autocomplete-devtools" => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: AUTOCOMPLETE_ID,
                    window_event: WindowEvent::Devtools,
                })
                .unwrap();
        },
        "update" => {
            tray_update(proxy);
        },
        "quit" => {
            proxy.send_event(Event::ControlFlow(ControlFlow::Exit)).unwrap();
        },
        "dashboard" => {
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
        "onboarding" => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID.clone(),
                    window_event: WindowEvent::Batch(vec![
                        WindowEvent::NavigateRelative {
                            path: LOGIN_PATH.into(),
                        },
                        WindowEvent::Show,
                    ]),
                })
                .unwrap();
        },
        "settings" => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID.clone(),
                    window_event: WindowEvent::Batch(vec![
                        WindowEvent::NavigateRelative {
                            path: "/autocomplete".into(),
                        },
                        WindowEvent::Show,
                    ]),
                })
                .unwrap();
        },
        "not-working" => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: DASHBOARD_ID.clone(),
                    window_event: WindowEvent::Batch(vec![
                        WindowEvent::NavigateRelative { path: "/help".into() },
                        WindowEvent::Show,
                    ]),
                })
                .unwrap();
        },
        "uninstall" => {
            tokio::runtime::Handle::current().spawn(async {
                fig_install::uninstall(InstallComponents::all()).await.ok();
                std::process::exit(0);
            });
        },
        "user-manual" => {
            if let Err(err) =
                fig_util::open_url("https://docs.aws.amazon.com/codewhisperer/latest/userguide/command-line.html")
            {
                error!(%err, "Failed to open user manual url")
            }
        },
        id => {
            for channel in Channel::all() {
                if id == format!("channel-{channel}") {
                    fig_settings::state::set_value("updates.channel", channel.to_string()).ok();
                    proxy.send_event(Event::ReloadTray).unwrap();
                    tray_update(proxy);
                    return;
                }
            }

            trace!(?id, "Unhandled tray event");
        },
    }

    tokio::spawn(fig_telemetry::send_menu_bar_actioned(Some(
        menu_event.id().0.to_owned(),
    )));
}

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
        #[cfg(not(target_os = "macos"))]
        let image = image::load_from_memory(include_bytes!("../icons/32x32.png"))
            .expect("Failed to open icon path")
            .into_rgba8();
        #[cfg(target_os = "macos")]
        let image = image::load_from_memory(include_bytes!("../icons/macos-menubar-template-icon@2x-scaled.png"))
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

pub fn build_tray(
    _event_loop_window_target: &EventLoopWindowTarget,
    _debug_state: &DebugState,
    _figterm_state: &FigtermState,
) -> tray_icon::Result<TrayIcon> {
    let tray_menu = get_context_menu();

    cfg_if!(
        if #[cfg(target_os = "linux")] {
            let icon_path = "/usr/share/icons/hicolor/64x64/apps/fig.png";
            let icon = load_icon(icon_path).unwrap_or_else(load_from_memory);
        } else {
            let icon = load_from_memory();
        }
    );

    #[allow(unused_mut)]
    let mut tray_builder = TrayIconBuilder::new().with_icon(icon).with_menu(Box::new(tray_menu));

    #[cfg(target_os = "macos")]
    {
        tray_builder = tray_builder.with_icon_as_template(true);
    }

    tray_builder.build()
}

pub fn get_context_menu() -> Menu {
    let mut tray_menu = Menu::new();

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
        // image_icon: Option<wry::application::window::Icon>,
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
        _image: Option<&'static [u8]>,
        text: impl Into<Cow<'static, str>>,
        id: impl Into<Cow<'static, str>>,
    ) -> Self {
        // cfg_if::cfg_if! {
        //     if #[cfg(target_os = "macos")] {
        //         let image_icon = match image {
        //             Some(image) => {
        //                 let image = image::load_from_memory(image)
        //                     .expect("Failed to open icon path")
        //                     .to_rgba8();

        //                 let (width, height) = image.dimensions();

        //                 wry::application::window::Icon::from_rgba(image.into_raw(), width, height).ok()
        //             },
        //             None => None,
        //         };
        //     } else {
        //         let _ = image;
        //         let image_icon = None;
        //     }
        // };

        Self::Entry {
            emoji_icon,
            // image_icon,
            text: text.into(),
            id: id.into(),
        }
    }

    // fn sub_menu(title: impl Into<Cow<'static, str>>, elements: Vec<MenuElement>) -> Self {
    //     Self::SubMenu {
    //         title: title.into(),
    //         elements,
    //     }
    // }

    fn add_to_menu(&self, menu: &mut Menu) {
        match self {
            MenuElement::Info(info) => {
                // menu.append(MenuItemAttributes::new(info).with_enabled(false));
                menu.append(&MenuItem::new(info, false, None)).unwrap();
            },
            MenuElement::Entry {
                emoji_icon, text, id, ..
            } => {
                let text = match (std::env::consts::OS, emoji_icon) {
                    ("linux", Some(emoji_icon)) => format!("{emoji_icon} {text}"),
                    _ => text.to_string(),
                };
                let menu_item = MenuItemBuilder::new()
                    .text(text)
                    .id(MenuId::new(id))
                    .enabled(true)
                    .build();
                menu.append(&menu_item).unwrap();
                // if let Some(image_icon) = &image_icon {
                //     custom_menu_item.set_icon(image_icon.clone());
                // }
            },
            MenuElement::Separator => {
                menu.append(&PredefinedMenuItem::separator()).unwrap();
            },
            MenuElement::SubMenu { title, elements } => {
                let sub_menu = Submenu::new(title, true);
                for element in elements {
                    element.add_to_submenu(&sub_menu);
                }

                menu.append(&sub_menu).unwrap();
            },
        }
    }

    fn add_to_submenu(&self, submenu: &Submenu) {
        match self {
            MenuElement::Info(info) => {
                // menu.append(MenuItemAttributes::new(info).with_enabled(false));
                submenu.append(&MenuItem::new(info, false, None)).unwrap();
            },
            MenuElement::Entry {
                emoji_icon, text, id, ..
            } => {
                let text: String = match (std::env::consts::OS, emoji_icon) {
                    ("linux", Some(emoji_icon)) => format!("{emoji_icon} {text}"),
                    _ => text.to_string(),
                };
                let menu_item = MenuItemBuilder::new()
                    .text(text)
                    .id(MenuId::new(id))
                    .enabled(true)
                    .build();
                submenu.append(&menu_item).unwrap();
                // if let Some(image_icon) = &image_icon {
                //     custom_menu_item.set_icon(image_icon.clone());
                // }
            },
            MenuElement::Separator => {
                submenu.append(&PredefinedMenuItem::separator()).unwrap();
            },
            MenuElement::SubMenu { title, elements } => {
                let sub_menu = Submenu::new(title, true);
                for element in elements {
                    element.add_to_submenu(&sub_menu);
                }

                submenu.append(&sub_menu).unwrap();
            },
        }
    }
}

fn menu() -> Vec<MenuElement> {
    // let logged_in = fig_request::auth::is_logged_in();
    let logged_in = true;

    let not_working = MenuElement::entry(None, None, "CW not working?", "not-working");
    let manual = MenuElement::entry(None, None, "User Guide", "user-manual");
    let version = MenuElement::Info(format!("Version: {}", env!("CARGO_PKG_VERSION")).into());
    let update = MenuElement::entry(None, None, "Check for updates...", "update");
    let quit = MenuElement::entry(None, None, "Quit CodeWhisperer", "quit");
    // let dashboard = MenuElement::entry(None, None, "Dashboard", "dashboard");
    let settings = MenuElement::entry(None, None, "Settings", "settings");
    // let developer = MenuElement::sub_menu("Developer", vec![
    //     MenuElement::entry(None, None, "Dashboard Devtools", "dashboard-devtools"),
    //     MenuElement::entry(None, None, "Autocomplete Devtools", "autocomplete-devtools"),
    //     MenuElement::entry(None, None, "Companion Devtools", "companion-devtools"),
    // ]);

    let mut menu = if !logged_in {
        vec![
            MenuElement::Info("CodeWhisperer hasn't been set up yet...".into()),
            MenuElement::entry(None, None, "Get Started", "/"),
            MenuElement::Separator,
            manual,
            not_working,
            MenuElement::Separator,
        ]
    } else {
        let mut menu = vec![];

        // accessibility not enabled
        // or shell integrations are not installed,
        // or input method not enabled AND kitty/alacritty/jetbrains installed

        // let handle = tokio::runtime::Handle::current();
        // let shell_not_installed = std::thread::spawn(move || {
        //     fig_util::Shell::all()
        //         .iter()
        //         .filter_map(|s| s.get_shell_integrations().ok())
        //         .flatten()
        //         .any(|i| handle.block_on(i.is_installed()).is_err())
        // })
        // .join()
        // .unwrap();

        // TOOD: renable, the lib is broken rn
        // let accessibility_not_installed = !PlatformState::accessibility_is_enabled().unwrap_or(true);

        // TODO: Add input method check

        // if accessibility_not_installed || shell_not_installed {
        //     menu.extend([
        //         MenuElement::Info("CodeWhisperer hasn't been configured correctly".into()),
        //         MenuElement::entry(None, None, "Fix Configuration Issues", "/help"),
        //         MenuElement::Separator,
        //     ]);
        // }

        menu.extend([
            settings,
            MenuElement::Separator,
            manual,
            not_working,
            MenuElement::Separator,
        ]);

        menu
    };

    menu.push(version);

    let max_channel = fig_install::get_max_channel();
    if max_channel != Channel::Stable {
        let channel = fig_install::get_channel().unwrap_or(Channel::Stable);

        menu.push(MenuElement::SubMenu {
            title: format!("Channel: {channel:#}").into(),
            elements: Channel::all()
                .iter()
                .filter_map(|c| {
                    if c > &max_channel {
                        None
                    } else if c == &channel {
                        Some(MenuElement::Info(format!("Channel: {c:#} (current)").into()))
                    } else {
                        Some(MenuElement::entry(
                            None,
                            None,
                            format!("Channel: {c:#}"),
                            format!("channel-{c}"),
                        ))
                    }
                })
                .collect(),
        });
    }

    menu.extend([update, MenuElement::Separator, quit]);

    menu
}
