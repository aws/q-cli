use std::borrow::Cow;

use tokio::sync::mpsc::UnboundedSender;
use tracing::{
    error,
    info,
    trace,
};
use x11rb::connection::Connection;
use x11rb::properties::WmClass;
use x11rb::protocol::xproto::{
    get_atom_name,
    get_geometry,
    get_input_focus,
    query_tree,
    ChangeWindowAttributesAux,
    ChangeWindowAttributesRequest,
    EventMask,
    Property,
    PropertyNotifyEvent,
    Window,
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

use crate::window::{
    CursorPositionKind,
    WindowEvent,
};

static WMCLASS_WHITELSIT: &[&str] = &["Gnome-terminal"];
pub const CURSOR_POSITION_KIND: CursorPositionKind = CursorPositionKind::Absolute;

#[derive(Debug)]
pub struct NativeState;

impl NativeState {
    pub fn new(window_event_sender: UnboundedSender<WindowEvent>) -> Self {
        tauri::async_runtime::spawn_blocking(move || handle_x11(window_event_sender));

        Self
    }
}

fn handle_x11(sender: UnboundedSender<WindowEvent>) {
    let (conn, screen_num) = x11rb::connect(None).expect("Failed to connect to X server");

    let setup = conn.setup();

    trace!(
        "connected to X{}.{} release {} screen number {screen_num}",
        setup.protocol_major_version,
        setup.protocol_minor_version,
        setup.release_number,
    );

    let screen = &setup.roots[screen_num];

    let request = ChangeWindowAttributesRequest {
        window: screen.root,
        value_list: Cow::Owned(ChangeWindowAttributesAux {
            event_mask: Some(u32::from(EventMask::PROPERTY_CHANGE)),
            ..Default::default()
        }),
    };

    request
        .send(&conn)
        .expect("Failed sending event mask update")
        .check()
        .expect("Failed changing event mask");

    while let Ok(event) = conn.wait_for_event() {
        if let Event::PropertyNotify(event) = event {
            if let Err(err) = handle_property_event(&conn, &sender, event) {
                error!("error handling PropertyNotifyEvent: {err}");
            }
        }
    }
}

fn handle_property_event(
    conn: &RustConnection,
    sender: &UnboundedSender<WindowEvent>,
    event: PropertyNotifyEvent,
) -> anyhow::Result<()> {
    let PropertyNotifyEvent { atom, state, .. } = event;
    if state == Property::NEW_VALUE {
        // TODO(mia): cache this
        let property_name = get_atom_name(conn, atom)?.reply()?.name;

        if property_name == b"_NET_ACTIVE_WINDOW" {
            trace!("active window changed");
            let focus = get_input_focus(conn)?.reply()?.focus;
            process_window(conn, sender, focus)?;
        }
    }

    Ok(())
}

fn process_window(conn: &RustConnection, sender: &UnboundedSender<WindowEvent>, window: Window) -> anyhow::Result<()> {
    let wm_class = match WmClass::get(conn, window)?.reply() {
        Ok(class_raw) => String::from_utf8_lossy(class_raw.class()).into_owned(),
        Err(_) => {
            // hide if missing wm class
            sender.send(WindowEvent::Hide)?;
            return Ok(());
        },
    };

    info!("focus changed to {wm_class}");

    if wm_class.as_str() == "Fig_tauri" {
        return Ok(());
    }

    if !WMCLASS_WHITELSIT.contains(&wm_class.as_str()) {
        // hide if not a whitelisted wm class
        sender.send(WindowEvent::Hide)?;
        return Ok(());
    }

    // TODO(mia): get the geometry and subscribe to changes

    // let mut frame = window;
    // let query = query_tree(conn, window)?.reply()?;
    // let root = query.root;
    // let mut parent = query.parent;

    // while parent != root {
    //     frame = parent;
    //     parent = query_tree(conn, frame)?.reply()?.parent;
    // }

    // let geometry = get_geometry(conn, frame)?.reply()?;

    // sender.send(WindowEvent::Reposition {
    //     x: geometry.x as i32,
    //     y: geometry.y as i32,
    // })?;

    sender.send(WindowEvent::Show)?;

    Ok(())
}
