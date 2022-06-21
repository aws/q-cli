use std::path::Path;

use cfg_if::cfg_if;
use tracing::trace;
use wry::application::event_loop::ControlFlow;
use wry::application::menu::{
    ContextMenu,
    MenuId,
    MenuItem,
    MenuItemAttributes,
};
use wry::application::system_tray::SystemTrayBuilder;
use wry::application::window::Icon;

use crate::event::{
    Event,
    WindowEvent,
};
use crate::{
    EventLoop,
    EventLoopProxy,
    GlobalState,
    AUTOCOMPLETE_ID,
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
        id => {
            trace!("Unhandled tray event: {id:?}");
        },
    }
}

// pub fn handle_tray_event(
//    app: &AppHandle,
//    event: SystemTrayEvent,
//    debug_state: Arc<DebugState>,
//    figterm_state: Arc<FigtermState>,
// ) {
//    match event {
//        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
//            "debugger-refresh" => {
//                if let Err(err) = update_tray_menu(app, &debug_state, &figterm_state) {
//                    warn!("Failed to update tray menu: {}", err);
//                }
//            },
//            "quit" => {
//                app.exit(0);
//            },
//            unknown_id => warn!("unknown menu item clicked: '{}'", unknown_id),
//        },
//        SystemTrayEvent::LeftClick { .. } | SystemTrayEvent::RightClick { .. } => {
//            if let Err(err) = update_tray_menu(app, &debug_state, &figterm_state) {
//                warn!("Failed to update tray menu: {}", err);
//            }
//        },
//        _ => {},
//    }
// }

fn load_icon(path: impl AsRef<Path>) -> Icon {
    let (icon_rgba, icon_width, icon_height) = {
        let image = image::open(path).expect("Failed to open icon path").into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to open icon")
}

pub fn build_tray(event_loop: &EventLoop, global_state: &GlobalState) -> wry::Result<()> {
    let mut tray_menu = ContextMenu::new();

    create_tray_menu(&mut tray_menu, global_state);

    cfg_if!(
        if #[cfg(target_os = "linux")] {
            let icon_path = "/usr/share/icons/hicolor/64x64/apps/fig.png";
        } else if #[cfg(target_os = "macos")] {
            // TODO: use transparent white icon
            let icon_path = ; // fix me!
        } else if #[cfg(target_os = "windows")] {
            let icon_path = ; // fix me!
        } else {
            compile_error!("Unsupported platform");
        }
    );

    let icon = load_icon(icon_path);

    SystemTrayBuilder::new(icon, Some(tray_menu)).build(event_loop)?;
    Ok(())
}

fn create_tray_menu(tray_menu: &mut ContextMenu, global_state: &GlobalState) {
    let figterm_session = global_state.figterm_state.most_recent_session();

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

    let api_message = format!("api-message: {}", match &*global_state.debug_state.debug_lines.read() {
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

    tray_menu.add_submenu("Debugger", true, debugger_menu);

    tray_menu.add_item(MenuItemAttributes::new("Toggle Devtools").with_id(MenuId::new("toggle-devtools")));

    tray_menu.add_item(MenuItemAttributes::new("Quit").with_id(MenuId::new("quit")));
}
