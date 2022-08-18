use std::sync::atomic::Ordering;
use std::sync::Arc;

use tracing::{
    debug,
    error,
    info,
    trace,
};
use x11rb::connection::Connection;
use x11rb::properties::WmClass;
use x11rb::protocol::xproto::{
    change_window_attributes,
    get_atom_name,
    get_geometry,
    get_input_focus,
    get_property,
    query_tree,
    AtomEnum,
    ChangeWindowAttributesAux,
    EventMask,
    GetGeometryReply,
    Property,
    PropertyNotifyEvent,
    Window,
};
use x11rb::protocol::Event as X11Event;
use x11rb::rust_connection::RustConnection;

use super::integrations::WM_CLASS_WHITELIST;
use super::{
    NativeState,
    WM_REVICED_DATA,
};
use crate::event::WindowEvent;
use crate::native::{
    WindowGeometry,
    X11WindowData,
};
use crate::{
    Event,
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

mod atoms {
    use once_cell::sync::OnceCell;
    use x11rb::protocol::xproto::{
        intern_atom,
        Atom,
    };
    use x11rb::rust_connection::RustConnection;

    static WM_ROLE: OnceCell<Atom> = OnceCell::new();

    pub(super) fn wm_role(conn: &RustConnection) -> Atom {
        *WM_ROLE.get_or_init(|| {
            intern_atom(conn, false, "WM_ROLE".as_bytes())
                .expect("Failed requesting WM_ROLE atom")
                .reply()
                .expect("Failed receiving WM_ROLE atom")
                .atom
        })
    }
}

pub async fn handle_x11(proxy: EventLoopProxy, native_state: Arc<NativeState>) {
    let (conn, screen_num) = x11rb::connect(None).expect("Failed to connect to X server");

    let setup = conn.setup();

    trace!(
        "connected to X{}.{} release {} screen number {screen_num}",
        setup.protocol_major_version,
        setup.protocol_minor_version,
        setup.release_number,
    );

    let screen = &setup.roots[screen_num];

    change_window_attributes(&conn, screen.root, &ChangeWindowAttributesAux {
        event_mask: Some(u32::from(EventMask::PROPERTY_CHANGE)),
        ..Default::default()
    })
    .expect("Failed sending event mask update")
    .check()
    .expect("Failed changing event mask");

    while let Ok(event) = tokio::task::block_in_place(|| conn.wait_for_event()) {
        if let X11Event::PropertyNotify(event) = event {
            if let Err(err) = handle_property_event(&conn, &native_state, &proxy, event) {
                error!("error handling PropertyNotifyEvent: {err}");
            }
        }
    }
}

fn handle_property_event(
    conn: &RustConnection,
    native_state: &NativeState,
    proxy: &EventLoopProxy,
    event: PropertyNotifyEvent,
) -> anyhow::Result<()> {
    WM_REVICED_DATA.store(true, Ordering::Relaxed);
    let PropertyNotifyEvent { atom, state, .. } = event;
    if state == Property::NEW_VALUE {
        // TODO(mia): cache this
        let property_name = get_atom_name(conn, atom)?.reply()?.name;

        if property_name == b"_NET_ACTIVE_WINDOW" {
            trace!("active window changed");
            process_window(conn, native_state, proxy)?;
        }
    }

    Ok(())
}

fn process_window(conn: &RustConnection, native_state: &NativeState, proxy: &EventLoopProxy) -> anyhow::Result<()> {
    let hide = || {
        proxy.send_event(Event::WindowEvent {
            window_id: AUTOCOMPLETE_ID.clone(),
            window_event: WindowEvent::Hide,
        })
    };

    let focus_window = get_input_focus(conn)?.reply()?.focus;
    trace!("Active window id: {focus_window}");

    if focus_window == 0 {
        hide()?;
        return Ok(());
    }

    let wm_class = WmClass::get(conn, focus_window)?.reply();

    let window_reply = window_geometry(conn, focus_window);

    let old_window_data = native_state.x11_active_window.lock().replace(X11WindowData {
        id: focus_window,
        class: wm_class.as_ref().ok().map(|wm_class| wm_class.class().to_owned()),
        instance: wm_class.as_ref().ok().map(|wm_class| wm_class.instance().to_owned()),
        window_geometry: window_reply.ok().map(|window_reply| WindowGeometry {
            x: window_reply.x as i32,
            y: window_reply.y as i32,
            width: window_reply.width as i32,
            height: window_reply.height as i32,
        }),
    });

    let wm_class = match wm_class {
        Ok(class_raw) => class_raw.class().to_owned(),
        Err(err) => {
            debug!("No wm class {err:?}");
            // hide if missing wm class
            hide()?;
            return Ok(());
        },
    };

    debug!("focus changed to {}", wm_class.escape_ascii());

    if wm_class == b"Fig_desktop" {
        // get wm_role
        let reply = get_property(
            conn,
            false,
            focus_window,
            atoms::wm_role(conn),
            AtomEnum::STRING,
            0,
            2048,
        )?
        .reply()?;

        if &reply.value != b"autocomplete" {
            // hide if not an autocomplete window
            hide()?;
        }

        return Ok(());
    }

    info!("Not autocomplete");

    if let Some(old_window_data) = old_window_data {
        if old_window_data.id != focus_window {
            hide()?;
            return Ok(());
        }
    }

    if !WM_CLASS_WHITELIST.keys().any(|w| w.as_bytes() == wm_class) {
        // hide if not a whitelisted wm class
        hide()?;
        return Ok(());
    }

    info!("Not whitelisted");

    // TODO(mia): get the geometry and subscribe to changes

    Ok(())
}

fn window_geometry(connection: &RustConnection, window: Window) -> anyhow::Result<GetGeometryReply> {
    let mut frame = window;
    let query = query_tree(connection, window)?.reply()?;
    let root = query.root;
    let mut parent = query.parent;

    while parent != root {
        frame = parent;
        parent = query_tree(connection, frame)?.reply()?.parent;
    }

    let geometry = get_geometry(connection, frame)?.reply()?;

    Ok(geometry)
}
