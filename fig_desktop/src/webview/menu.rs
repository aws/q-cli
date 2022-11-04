use once_cell::sync::Lazy;
use wry::application::menu::{
    MenuBar,
    MenuId,
    MenuItem,
};

use crate::event::{
    Event,
    WindowEvent,
};
use crate::{
    EventLoopProxy,
    DASHBOARD_ID,
};

static DASHBOARD_QUIT: Lazy<MenuId> = Lazy::new(|| MenuId::new("dashboard-quit"));
static DASHBOARD_RELOAD: Lazy<MenuId> = Lazy::new(|| MenuId::new("dashboard-reload"));

#[cfg(target_os = "macos")]
pub fn menu_bar() -> MenuBar {
    use wry::application::accelerator::Accelerator;
    use wry::application::keyboard::{
        KeyCode,
        ModifiersState,
    };
    use wry::application::menu::MenuItemAttributes;

    let mut menu_bar = MenuBar::new();

    let mut app_submenu = MenuBar::new();
    app_submenu.add_native_item(MenuItem::CloseWindow);
    app_submenu.add_item(
        MenuItemAttributes::new("Quit Fig (UI)")
            .with_accelerators(&Accelerator::new(ModifiersState::SUPER, KeyCode::KeyQ))
            .with_id(*DASHBOARD_QUIT),
    );
    app_submenu.add_item(
        MenuItemAttributes::new("Reload")
            .with_accelerators(&Accelerator::new(ModifiersState::SUPER, KeyCode::KeyR))
            .with_id(*DASHBOARD_RELOAD),
    );

    menu_bar.add_submenu("Fig", true, app_submenu);

    let mut edit_submenu = MenuBar::new();

    edit_submenu.add_native_item(MenuItem::Undo);
    edit_submenu.add_native_item(MenuItem::Redo);
    edit_submenu.add_native_item(MenuItem::Separator);
    edit_submenu.add_native_item(MenuItem::Copy);
    edit_submenu.add_native_item(MenuItem::Paste);
    edit_submenu.add_native_item(MenuItem::Cut);
    edit_submenu.add_native_item(MenuItem::SelectAll);

    menu_bar.add_submenu("Edit", true, edit_submenu);

    menu_bar
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

    menu_bar.add_submenu("Fig", true, app_submenu);

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

#[cfg(target_os = "linux")]
pub fn menu_bar() -> MenuBar {
    let mut menu_bar = MenuBar::new();

    let mut app_submenu = MenuBar::new();
    app_submenu.add_native_item(MenuItem::Hide);
    app_submenu.add_native_item(MenuItem::HideOthers);
    app_submenu.add_native_item(MenuItem::ShowAll);
    app_submenu.add_native_item(MenuItem::Separator);
    app_submenu.add_native_item(MenuItem::Quit);

    menu_bar.add_submenu("App", true, app_submenu);

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

pub fn handle_event(menu_id: MenuId, proxy: &EventLoopProxy) {
    match menu_id {
        menu_id if menu_id == *DASHBOARD_QUIT => proxy
            .send_event(Event::WindowEvent {
                window_id: DASHBOARD_ID,
                window_event: WindowEvent::Hide,
            })
            .unwrap(),
        menu_id if menu_id == *DASHBOARD_RELOAD => proxy
            .send_event(Event::WindowEvent {
                window_id: DASHBOARD_ID,
                window_event: WindowEvent::Reload,
            })
            .unwrap(),
        _ => {},
    }
}
