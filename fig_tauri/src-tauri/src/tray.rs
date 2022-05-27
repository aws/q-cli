use wry::application::event_loop::{
    ControlFlow,
    EventLoop,
    EventLoopProxy,
};
use wry::application::menu::{
    ContextMenu,
    CustomMenuItem,
    MenuId,
    MenuItem,
    MenuItemAttributes,
};
use wry::application::system_tray::SystemTrayBuilder;

use crate::window::FigWindowEvent;
use crate::{
    FigEvent,
    AUTOCOMPLETE_ID,
};

struct TrayElement {
    item: CustomMenuItem,
    event: Box<dyn Fn(&EventLoopProxy<FigEvent>)>,
}

pub struct Tray {
    elements: Vec<TrayElement>,
}

impl Tray {
    pub fn handle_event(&self, id: MenuId, proxy: &EventLoopProxy<FigEvent>) {
        for TrayElement { item, event } in &self.elements {
            if item.clone().id() == id {
                event(proxy);
            }
        }
    }
}

pub fn create_tray(event_loop: &EventLoop<FigEvent>) -> wry::Result<Tray> {
    let mut tray_menu = ContextMenu::new();
    let elements = create_tray_menu(&mut tray_menu);
    SystemTrayBuilder::new("/usr/share/icons/hicolor/32x32/apps/fig.png".into(), Some(tray_menu)).build(event_loop)?;
    Ok(Tray { elements })
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

fn create_tray_menu(tray_menu: &mut ContextMenu) -> Vec<TrayElement> {
    let mut v = vec![];

    let mut debugger_menu = ContextMenu::new();
    debugger_menu.add_item(
        MenuItemAttributes::new("Fig can't link your terminal window to the TTY")
            .with_id(MenuId::new("debugger-status"))
            .with_enabled(false),
    );
    debugger_menu.add_native_item(MenuItem::Separator);
    debugger_menu.add_item(
        MenuItemAttributes::new("window: None")
            .with_id(MenuId::new("debugger-window"))
            .with_enabled(false),
    );
    debugger_menu.add_item(
        MenuItemAttributes::new("tty: None")
            .with_id(MenuId::new("debugger-tty"))
            .with_enabled(false),
    );
    debugger_menu.add_item(
        MenuItemAttributes::new("cwd: None")
            .with_id(MenuId::new("debugger-cwd"))
            .with_enabled(false),
    );
    debugger_menu.add_item(
        MenuItemAttributes::new("pid: None")
            .with_id(MenuId::new("debugger-pid"))
            .with_enabled(false),
    );
    debugger_menu.add_item(
        MenuItemAttributes::new("keybuffer: None")
            .with_id(MenuId::new("debugger-keybuffer"))
            .with_enabled(false),
    );
    debugger_menu.add_item(
        MenuItemAttributes::new("hostname: None")
            .with_id(MenuId::new("debugger-hostname"))
            .with_enabled(false),
    );
    debugger_menu.add_item(
        MenuItemAttributes::new("terminal: None")
            .with_id(MenuId::new("debugger-terminal"))
            .with_enabled(false),
    );
    debugger_menu.add_item(
        MenuItemAttributes::new("process: None")
            .with_id(MenuId::new("debugger-process"))
            .with_enabled(false),
    );
    debugger_menu.add_item(
        MenuItemAttributes::new("api-message: None")
            .with_id(MenuId::new("debugger-api-message"))
            .with_enabled(false),
    );
    debugger_menu.add_native_item(MenuItem::Separator);
    debugger_menu.add_item(MenuItemAttributes::new("Manually Refresh Menu").with_id(MenuId::new("debugger-refresh")));

    tray_menu.add_submenu("Debugger", true, debugger_menu);

    v.push(TrayElement {
        item: tray_menu.add_item(MenuItemAttributes::new("Toggle Devtools").with_id(MenuId::new("toggle-devtools"))),
        event: Box::new(|proxy| {
            proxy
                .send_event(FigEvent::WindowEvent {
                    fig_id: AUTOCOMPLETE_ID,
                    window_event: FigWindowEvent::Devtools,
                })
                .unwrap();
        }),
    });

    v.push(TrayElement {
        item: tray_menu.add_item(MenuItemAttributes::new("Quit").with_id(MenuId::new("quit"))),
        event: Box::new(|proxy| {
            proxy.send_event(FigEvent::ControlFlow(ControlFlow::Exit)).unwrap();
        }),
    });

    v
}

// fn update_tray_menu(debug_state: &DebugState, figterm_state: &FigtermState) -> Result<(),
// tauri::Error> {
//
//    let figterm_session = figterm_state.most_recent_session();
//
//    match figterm_session {
//        Some(_) => {
//            app.tray_handle()
//                .get_item("debugger-status")
//                .set_title("Fig is running as expected")?;
//        },
//        None => {
//            app.tray_handle()
//                .get_item("debugger-status")
//                .set_title("Fig can't link your terminal window to the TTY")?;
//        },
//    }
//
//    macro_rules! context_debugger {
//        ($menu_elem:expr, $fmt_str:expr, $func:ident) => {{
//            let tty_text = format!(
//                $fmt_str,
//                match figterm_session.as_ref().and_then(|session| session.context.as_ref()) {
//                    Some(context) => context.$func().to_string().trim().to_string(),
//                    None => "None".to_string(),
//                }
//            );
//
//            app.tray_handle().get_item($menu_elem).set_title(tty_text)?;
//        }};
//    }
//
//    context_debugger!("debugger-tty", "tty: {}", ttys);
//    context_debugger!("debugger-cwd", "cwd: {}", current_working_directory);
//    context_debugger!("debugger-pid", "pid: {}", pid);
//    context_debugger!("debugger-hostname", "hostname: {}", hostname);
//    context_debugger!("debugger-terminal", "terminal: {}", terminal);
//    context_debugger!("debugger-process", "process: {}", process_name);
//
//    let keybuffer_text = format!("keybuffer: {}", match figterm_session.as_ref() {
//        Some(session) => {
//            let mut edit_buffer = session.edit_buffer.text.clone();
//            if let Ok(cursor) = session.edit_buffer.cursor.try_into() {
//                edit_buffer.insert(cursor, '|');
//            }
//            edit_buffer
//        },
//        None => "None".to_string(),
//    });
//
//    app.tray_handle()
//        .get_item("debugger-keybuffer")
//        .set_title(keybuffer_text)?;
//
//    let api_message = format!("api-message: {}", match &*debug_state.debug_lines.read() {
//        v if !v.is_empty() => v.join(" | "),
//        _ => "None".to_string(),
//    });
//
//    app.tray_handle()
//        .get_item("debugger-api-message")
//        .set_title(api_message)?;
//
//    trace!("Updating tray menu");
//
//    Ok(())
//}
