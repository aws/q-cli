use std::sync::Arc;

use tauri::{
    AppHandle, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
    SystemTraySubmenu,
};
use tracing::{trace, warn};

use crate::{figterm::FigtermState, DebugState};

pub fn create_tray() -> SystemTray {
    SystemTray::new().with_menu(create_tray_menu())
}

pub fn handle_tray_event(
    app: &AppHandle,
    event: SystemTrayEvent,
    debug_state: Arc<DebugState>,
    figterm_state: Arc<FigtermState>,
) {
    match event {
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "debugger-refresh" => {
                if let Err(err) = update_tray_menu(app, &debug_state, &figterm_state) {
                    warn!("Failed to update tray menu: {}", err);
                }
            }
            "quit" => {
                app.exit(0);
            }
            unknown_id => warn!("unknown menu item clicked: '{}'", unknown_id),
        },
        SystemTrayEvent::LeftClick { .. } | SystemTrayEvent::RightClick { .. } => {
            if let Err(err) = update_tray_menu(app, &debug_state, &figterm_state) {
                warn!("Failed to update tray menu: {}", err);
            }
        }
        _ => {}
    }
}

fn create_tray_menu() -> SystemTrayMenu {
    SystemTrayMenu::new()
        .add_submenu(SystemTraySubmenu::new(
            "Debugger",
            SystemTrayMenu::new()
                .add_item(CustomMenuItem::new(
                    "debugger-status",
                    "Fig can't link your terminal window to the TTY",
                ))
                .add_native_item(SystemTrayMenuItem::Separator)
                .add_item(CustomMenuItem::new("debugger-window", "window: None").disabled())
                .add_item(CustomMenuItem::new("debugger-tty", "tty: None").disabled())
                .add_item(CustomMenuItem::new("debugger-cwd", "cwd: None").disabled())
                .add_item(CustomMenuItem::new("debugger-pid", "pid: None").disabled())
                .add_item(CustomMenuItem::new("debugger-keybuffer", "keybuffer: None").disabled())
                .add_item(CustomMenuItem::new("debugger-hostname", "hostname: None").disabled())
                .add_item(CustomMenuItem::new("debugger-terminal", "terminal: None").disabled())
                .add_item(CustomMenuItem::new("debugger-process", "process: None").disabled())
                .add_item(
                    CustomMenuItem::new("debugger-api-message", "api-message: None").disabled(),
                )
                .add_native_item(SystemTrayMenuItem::Separator)
                .add_item(CustomMenuItem::new(
                    "debugger-refresh",
                    "Manually Refresh Menu",
                )),
        ))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit", "Quit"))
}

fn update_tray_menu(
    app: &AppHandle,
    debug_state: &DebugState,
    figterm_state: &FigtermState,
) -> Result<(), tauri::Error> {
    let figterm_session = figterm_state.most_recent_session();

    match figterm_session {
        Some(_) => {
            app.tray_handle()
                .get_item("debugger-status")
                .set_title("Fig is running as expected")?;
        }
        None => {
            app.tray_handle()
                .get_item("debugger-status")
                .set_title("Fig can't link your terminal window to the TTY")?;
        }
    }

    macro_rules! context_debugger {
        ($menu_elem:expr, $fmt_str:expr, $func:ident) => {{
            let tty_text = format!(
                $fmt_str,
                match figterm_session
                    .as_ref()
                    .and_then(|session| session.context.as_ref())
                {
                    Some(context) => context.$func().to_string().trim().to_string(),
                    None => "None".to_string(),
                }
            );

            app.tray_handle().get_item($menu_elem).set_title(tty_text)?;
        }};
    }

    context_debugger!("debugger-tty", "tty: {}", ttys);
    context_debugger!("debugger-cwd", "cwd: {}", current_working_directory);
    context_debugger!("debugger-pid", "pid: {}", pid);
    context_debugger!("debugger-hostname", "hostname: {}", hostname);
    context_debugger!("debugger-terminal", "terminal: {}", terminal);
    context_debugger!("debugger-process", "process: {}", process_name);

    let keybuffer_text = format!(
        "keybuffer: {}",
        match figterm_session.as_ref() {
            Some(session) => {
                let mut edit_buffer = session.edit_buffer.text.clone();
                if let Ok(cursor) = session.edit_buffer.cursor.try_into() {
                    edit_buffer.insert(cursor, '|');
                }
                edit_buffer
            }
            None => "None".to_string(),
        }
    );

    app.tray_handle()
        .get_item("debugger-keybuffer")
        .set_title(keybuffer_text)?;

    let api_message = format!(
        "api-message: {}",
        match &*debug_state.debug_lines.read() {
            v if !v.is_empty() => v.join(" | "),
            _ => "None".to_string(),
        }
    );

    app.tray_handle()
        .get_item("debugger-api-message")
        .set_title(api_message)?;

    trace!("Updating tray menu");

    Ok(())
}
