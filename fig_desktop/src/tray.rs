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
    EventLoop,
    EventLoopProxy,
    AUTOCOMPLETE_ID,
    MISSION_CONTROL_ID,
};

pub fn handle_event(id: MenuId, proxy: &EventLoopProxy) {
    match id {
        id if id == MenuId::new("debugger-refresh") => {
            proxy.send_event(Event::RefreshDebugger).unwrap();
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
        id if id == MenuId::new("dashboard") => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: MISSION_CONTROL_ID,
                    window_event: WindowEvent::Show,
                })
                .unwrap();
        },
        id if id == MenuId::new("settings") => {
            proxy
                .send_event(Event::WindowEvent {
                    window_id: MISSION_CONTROL_ID,
                    window_event: WindowEvent::Show,
                })
                .unwrap();
            proxy
                .send_event(Event::WindowEvent {
                    window_id: MISSION_CONTROL_ID,
                    window_event: WindowEvent::NatigateRelative {
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

#[cfg(not(target_os = "linux"))]
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

pub fn build_tray(
    event_loop: &EventLoop,
    debug_state: &DebugState,
    figterm_state: &FigtermState,
) -> wry::Result<SystemTray> {
    let mut tray_menu = ContextMenu::new();

    create_tray_menu(&mut tray_menu, debug_state, figterm_state);

    cfg_if!(
        if #[cfg(target_os = "linux")] {
            let icon_path = "/usr/share/icons/hicolor/64x64/apps/fig.png";
            let icon = load_icon(icon_path);
        } else {
            let icon = load_from_memory();
        }
    );

    Ok(SystemTrayBuilder::new(icon, Some(tray_menu)).build(event_loop)?)
}

fn create_tray_menu(tray_menu: &mut ContextMenu, debug_state: &DebugState, figterm_state: &FigtermState) {
    let figterm_session = figterm_state.with_most_recent(|session| session.get_info());

    // Debugger Menu

    let debugger_status = match figterm_session {
        Some(_) => "Fig is running as expected",
        None => "Fig can't link your terminal window to the TTY",
    };

    let mut debugger_menu = ContextMenu::new();
    debugger_menu.add_item(
        MenuItemAttributes::new(debugger_status)
            .with_id(MenuId::new("debugger-status"))
            .with_enabled(false),
    );

    debugger_menu.add_native_item(MenuItem::Separator);

    // Debugger Menu Elements

    macro_rules! context_debugger {
        ($menu_elem:expr, $fmt_str:expr, $func:ident) => {{
            let tty_text = format!(
                $fmt_str,
                match figterm_session.as_ref().and_then(|session| session.context.as_ref()) {
                    Some(context) => context.$func().to_string().trim().to_string(),
                    None => "None".to_string(),
                }
            );

            debugger_menu.add_item(
                MenuItemAttributes::new(&tty_text)
                    .with_id(MenuId::new($menu_elem.into()))
                    .with_enabled(false),
            );
        }};
    }

    context_debugger!("debugger-tty", "tty: {}", ttys);
    context_debugger!("debugger-cwd", "cwd: {}", current_working_directory);
    context_debugger!("debugger-pid", "pid: {}", pid);

    let keybuffer_text = format!("keybuffer: {}", match figterm_session.as_ref() {
        Some(session) => {
            let mut edit_buffer = session.edit_buffer.text.clone();
            if let Ok(cursor) = session.edit_buffer.cursor.try_into() {
                edit_buffer.insert(cursor, '|');
            }
            edit_buffer
        },
        None => "None".to_string(),
    });

    debugger_menu.add_item(
        MenuItemAttributes::new(&keybuffer_text)
            .with_id(MenuId::new("debugger-keybuffer"))
            .with_enabled(false),
    );

    context_debugger!("debugger-hostname", "hostname: {}", hostname);
    context_debugger!("debugger-terminal", "terminal: {}", terminal);
    context_debugger!("debugger-process", "process: {}", process_name);

    let api_message = format!("api-message: {}", match &*debug_state.debug_lines.read() {
        v if !v.is_empty() => v.join(" | "),
        _ => "None".to_string(),
    });

    debugger_menu.add_item(
        MenuItemAttributes::new(&api_message)
            .with_id(MenuId::new("debugger-api-message"))
            .with_enabled(false),
    );

    debugger_menu.add_native_item(MenuItem::Separator);

    debugger_menu.add_item(MenuItemAttributes::new("Manually Refresh Menu").with_id(MenuId::new("debugger-refresh")));

    tray_menu.add_item(MenuItemAttributes::new(&menu_name("🎛️", "Dashboard")).with_id(MenuId::new("dashboard")));

    tray_menu.add_item(MenuItemAttributes::new(&menu_name("⚙️", "Settings")).with_id(MenuId::new("settings")));

    tray_menu.add_native_item(MenuItem::Separator);

    tray_menu.add_item(MenuItemAttributes::new(&menu_name("📚", "User Manual")).with_id(MenuId::new("user-manual")));

    tray_menu.add_item(MenuItemAttributes::new(&menu_name("💬", "Join Community")).with_id(MenuId::new("community")));

    tray_menu.add_native_item(MenuItem::Separator);

    tray_menu.add_item(MenuItemAttributes::new(&menu_name("🐞", "Report an Issue")).with_id(MenuId::new("issue")));

    tray_menu.add_native_item(MenuItem::Separator);

    tray_menu.add_submenu("Debugger", true, debugger_menu);

    tray_menu.add_item(
        MenuItemAttributes::new(&format!("Version {}", env!("CARGO_PKG_VERSION"))).with_id(MenuId::new("version")),
    );

    tray_menu
        .add_item(MenuItemAttributes::new(&menu_name("", "Toggle Devtools")).with_id(MenuId::new("toggle-devtools")));

    tray_menu.add_native_item(MenuItem::Separator);

    tray_menu.add_item(MenuItemAttributes::new(&menu_name("❌", "Quit")).with_id(MenuId::new("quit")));
}

fn menu_name(icon: &str, name: &str) -> String {
    if std::env::consts::OS == "windows" {
        name.into()
    } else {
        format!("{icon} {name}")
    }
}
