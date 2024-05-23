use fig_util::consts::PRODUCT_NAME;
use muda::{
    Menu,
    MenuEvent,
    Submenu,
};

use crate::event::{
    Event,
    WindowEvent,
};
use crate::{
    EventLoopProxy,
    DASHBOARD_ID,
    HOTKEY_CHAT_ID,
};

const DASHBOARD_QUIT: &str = "dashboard-quit";
const DASHBOARD_RELOAD: &str = "dashboard-reload";
const DASHBOARD_BACK: &str = "dashboard-back";
const DASHBOARD_FORWARD: &str = "dashboard-forward";

const HOTKEY_DEVTOOLS: &str = "hotkey-devtools";
const HOTKEY_ENTIRE_SCREEN: &str = "hotkey-entire-screen";

#[cfg(target_os = "macos")]
pub fn menu_bar() -> Menu {
    use muda::{
        MenuItemBuilder,
        PredefinedMenuItem,
        Submenu,
    };

    let menu_bar = Menu::new();

    let app_submenu = Submenu::new(PRODUCT_NAME, true);
    app_submenu
        .append_items(&[
            &MenuItemBuilder::new()
                .text("Backward")
                .id(DASHBOARD_BACK.into())
                .enabled(true)
                .acccelerator(Some("super+["))
                .unwrap()
                .build(),
            &MenuItemBuilder::new()
                .text("Forward")
                .id(DASHBOARD_FORWARD.into())
                .enabled(true)
                .acccelerator(Some("super+]"))
                .unwrap()
                .build(),
            &MenuItemBuilder::new()
                .text("Reload")
                .id(DASHBOARD_RELOAD.into())
                .enabled(true)
                .acccelerator(Some("super+r"))
                .unwrap()
                .build(),
            &MenuItemBuilder::new()
                .text("Close Window")
                .id(DASHBOARD_QUIT.into())
                .enabled(true)
                .acccelerator(Some("super+w"))
                .unwrap()
                .build(),
            &MenuItemBuilder::new()
                .text(format!("Quit {PRODUCT_NAME} (UI)"))
                .id(DASHBOARD_QUIT.into())
                .enabled(true)
                .acccelerator(Some("super+q"))
                .unwrap()
                .build(),
        ])
        .unwrap();

    menu_bar.append(&app_submenu).unwrap();

    let edit_submenu = Submenu::new("Edit", true);
    edit_submenu
        .append_items(&[
            &PredefinedMenuItem::undo(Some("Undo")),
            &PredefinedMenuItem::redo(Some("Redo")),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::copy(Some("Copy")),
            &PredefinedMenuItem::paste(Some("Paste")),
            &PredefinedMenuItem::cut(Some("Cut")),
            &PredefinedMenuItem::select_all(Some("Select All")),
        ])
        .unwrap();

    menu_bar.append(&edit_submenu).unwrap();

    let window_submenu = Submenu::new("Window", true);
    window_submenu
        .append_items(&[&PredefinedMenuItem::minimize(Some("Minimize"))])
        .unwrap();
    menu_bar.append(&window_submenu).unwrap();

    let help_submenu = Submenu::new("Help", true);
    menu_bar.append(&help_submenu).unwrap();

    menu_bar
}

#[cfg(target_os = "macos")]
pub fn context_menu() -> (Menu, Submenu) {
    use muda::{
        MenuItemBuilder,
        PredefinedMenuItem,
    };

    let menu_bar = Menu::new();
    let context_menu = Submenu::new("Context Menu", true);
    menu_bar.append(&context_menu).unwrap();

    context_menu
        .append_items(&[
            &PredefinedMenuItem::separator(),
            &MenuItemBuilder::new()
                .text("Entire Screen")
                .id(HOTKEY_ENTIRE_SCREEN.into())
                .enabled(true)
                .build(),
            &PredefinedMenuItem::separator(),
            &MenuItemBuilder::new()
                .text("Inspect Element")
                .id(HOTKEY_DEVTOOLS.into())
                .enabled(true)
                .build(),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::select_all(None),
        ])
        .unwrap();

    (menu_bar, context_menu)
}

// TODO(chay): add whatever is ergonomic for Windows
#[cfg(target_os = "windows")]
pub fn menu_bar() -> MenuBar {
    let mut menu_bar = MenuBar::new();

    let mut app_submenu = MenuBar::new();
    app_submenu.add_native_item(MenuItem::Hide);
    app_submenu.add_native_item(MenuItem::HideOthers);
    app_submenu.add_native_item(MenuItem::ShowAll);
    app_submenu.add_native_item(MenuItem::Separator);
    app_submenu.add_native_item(MenuItem::CloseWindow);
    app_submenu.add_native_item(MenuItem::Quit);

    menu_bar.add_submenu(PRODUCT_NAME, true, app_submenu);

    let mut edit_submenu = MenuBar::new();

    edit_submenu.add_native_item(MenuItem::Undo);
    edit_submenu.add_native_item(MenuItem::Redo);
    edit_submenu.add_native_item(MenuItem::Separator);
    edit_submenu.add_native_item(MenuItem::Cut);
    edit_submenu.add_native_item(MenuItem::Copy);
    edit_submenu.add_native_item(MenuItem::Paste);
    edit_submenu.add_native_item(MenuItem::Paste);
    edit_submenu.add_native_item(MenuItem::SelectAll);

    menu_bar.add_submenu("Edit", true, edit_submenu);

    menu_bar
}

pub fn handle_event(menu_event: &MenuEvent, proxy: &EventLoopProxy) {
    match &menu_event.id().0 {
        menu_id if menu_id == DASHBOARD_QUIT => proxy
            .send_event(Event::WindowEvent {
                window_id: DASHBOARD_ID,
                window_event: WindowEvent::Hide,
            })
            .unwrap(),
        menu_id if menu_id == DASHBOARD_RELOAD => proxy
            .send_event(Event::WindowEvent {
                window_id: DASHBOARD_ID,
                window_event: WindowEvent::Reload,
            })
            .unwrap(),
        menu_id if menu_id == DASHBOARD_BACK => proxy
            .send_event(Event::WindowEvent {
                window_id: DASHBOARD_ID,
                window_event: WindowEvent::NavigateBack,
            })
            .unwrap(),
        menu_id if menu_id == DASHBOARD_FORWARD => proxy
            .send_event(Event::WindowEvent {
                window_id: DASHBOARD_ID,
                window_event: WindowEvent::NavigateForward,
            })
            .unwrap(),
        menu_id if menu_id == HOTKEY_DEVTOOLS => proxy
            .send_event(Event::WindowEvent {
                window_id: HOTKEY_CHAT_ID,
                window_event: WindowEvent::Devtools,
            })
            .unwrap(),
        menu_id if menu_id == HOTKEY_ENTIRE_SCREEN => proxy
            .send_event(Event::WindowEvent {
                window_id: HOTKEY_CHAT_ID,
                window_event: WindowEvent::Event {
                    event_name: "screenshot".into(),
                    payload: None,
                },
            })
            .unwrap(),
        menu_id => proxy
            .send_event(Event::WindowEvent {
                window_id: HOTKEY_CHAT_ID,
                window_event: WindowEvent::Event {
                    event_name: "screenshot".into(),
                    payload: Some(serde_json::json!(menu_id).to_string().into()),
                },
            })
            .unwrap(),
    }
}
