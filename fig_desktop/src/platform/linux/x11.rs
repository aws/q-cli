use std::sync::atomic::Ordering;
use std::sync::Arc;

use dashmap::DashMap;
use parking_lot::Mutex;
use tracing::{
    debug,
    error,
    trace,
};
use wry::application::dpi::{
    PhysicalPosition,
    PhysicalSize,
};
use x11rb::connection::Connection;
use x11rb::properties::WmClass;
use x11rb::protocol::xproto::{
    change_window_attributes,
    get_atom_name,
    get_geometry,
    get_input_focus,
    get_property,
    intern_atom,
    query_tree,
    Atom,
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
    PlatformStateImpl,
    WM_REVICED_DATA,
};
use crate::event::WindowEvent;
use crate::utils::Rect;
use crate::{
    Event,
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

const WM_WINDOW_ROLE: &[u8] = b"WM_WINDOW_ROLE";

#[derive(Debug, Default)]
pub struct X11State {
    pub active_window: Mutex<Option<X11WindowData>>,
    pub atom_cache_a2b: DashMap<Atom, Vec<u8>>,
    pub atom_cache_b2a: DashMap<Vec<u8>, Atom>,
}

impl X11State {
    fn atom_a2b(&self, conn: &RustConnection, atom: Atom) -> anyhow::Result<Vec<u8>> {
        match self.atom_cache_a2b.get(&atom) {
            Some(name) => Ok(name.clone()),
            None => {
                let name = get_atom_name(conn, atom)?.reply()?.name;
                self.atom_cache_a2b.insert(atom, name.clone());
                Ok(name)
            },
        }
    }

    fn atom_b2a(&self, conn: &RustConnection, name: &[u8]) -> anyhow::Result<Atom> {
        match self.atom_cache_b2a.get(name) {
            Some(atom) => Ok(*atom),
            None => {
                let atom = intern_atom(conn, false, name)?.reply()?.atom;
                self.atom_cache_b2a.insert(name.to_vec(), atom);
                Ok(atom)
            },
        }
    }
}

#[derive(Debug)]
pub struct X11WindowData {
    pub id: x11rb::protocol::xproto::Window,
    pub class: Option<Vec<u8>>,
    pub instance: Option<Vec<u8>>,
    pub window_geometry: Option<Rect>,
}

pub(super) async fn handle_x11(
    proxy: EventLoopProxy,
    x11_state: Arc<X11State>,
    platform_state: Arc<PlatformStateImpl>,
) {
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
        event_mask: Some(EventMask::PROPERTY_CHANGE),
        ..Default::default()
    })
    .expect("Failed sending event mask update")
    .check()
    .expect("Failed changing event mask");

    while let Ok(event) = tokio::task::block_in_place(|| conn.wait_for_event()) {
        if let X11Event::PropertyNotify(event) = event {
            if let Err(err) = handle_property_event(&conn, &x11_state, &proxy, event, &platform_state) {
                error!("error handling PropertyNotifyEvent: {err}");
            }
        }
    }
}

fn handle_property_event(
    conn: &RustConnection,
    x11_state: &X11State,
    proxy: &EventLoopProxy,
    event: PropertyNotifyEvent,
    platform_state: &Arc<PlatformStateImpl>,
) -> anyhow::Result<()> {
    WM_REVICED_DATA.store(true, Ordering::Relaxed);
    let PropertyNotifyEvent { atom, state, .. } = event;
    if state == Property::NEW_VALUE {
        let property_name = x11_state.atom_a2b(conn, atom)?;

        if property_name == b"_NET_ACTIVE_WINDOW" {
            trace!("active window changed");
            process_window(conn, x11_state, proxy, platform_state)?;
        }
    }

    Ok(())
}

fn process_window(
    conn: &RustConnection,
    x11_state: &X11State,
    proxy: &EventLoopProxy,
    platform_state: &Arc<PlatformStateImpl>,
) -> anyhow::Result<()> {
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

    let old_window_data = x11_state.active_window.lock().replace(X11WindowData {
        id: focus_window,
        class: wm_class.as_ref().ok().map(|wm_class| wm_class.class().to_owned()),
        instance: wm_class.as_ref().ok().map(|wm_class| wm_class.instance().to_owned()),
        window_geometry: window_reply.ok().map(|window_reply| Rect {
            position: PhysicalPosition {
                x: window_reply.x,
                y: window_reply.y,
            }
            .into(),
            size: PhysicalSize {
                width: window_reply.width,
                height: window_reply.height,
            }
            .into(),
        }),
    });

    let wm_class = String::from_utf8_lossy(&match wm_class {
        Ok(class_raw) => class_raw.class().to_owned(),
        Err(err) => {
            debug!("No wm class {err:?}");
            // hide if missing wm class
            hide()?;
            return Ok(());
        },
    })
    .to_string();

    debug!("focus changed to {}", wm_class);

    if wm_class == "Fig" {
        // get wm_role
        let reply = get_property(
            conn,
            false,
            focus_window,
            x11_state.atom_b2a(conn, WM_WINDOW_ROLE)?,
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

    debug!("Selected window is not Fig");

    if let Some(terminal) = WM_CLASS_WHITELIST.get(&wm_class.as_str()) {
        *platform_state.active_terminal.lock() = Some(terminal.clone());
    }

    if let Some(old_window_data) = old_window_data {
        if old_window_data.id != focus_window {
            hide()?;
            return Ok(());
        }
    }

    if !WM_CLASS_WHITELIST.contains_key(&wm_class.as_str()) {
        hide()?;
        return Ok(());
    }

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
