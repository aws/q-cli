use std::borrow::Cow;
use std::path::Path;

use anyhow::Result;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
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
};
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;

use crate::window::WindowEvent;

static WMCLASS_WHITELSIT: &[&str] = &["Gnome-terminal"];

#[derive(Debug)]
pub struct NativeState;

impl NativeState {
    pub fn new(window_event_sender: UnboundedSender<WindowEvent>) -> Self {
        match DisplayServer::detect() {
            Ok(DisplayServer::X11) => {
                info!("Detected X11 server");
                tauri::async_runtime::spawn_blocking(move || handle_x11(window_event_sender));
            },
            Ok(DisplayServer::Wayland) => {
                info!("Detected Wayland server");
                if let Ok(sway_socket) = std::env::var("SWAYSOCK") {
                    info!("Using sway socket: {}", sway_socket);
                    tauri::async_runtime::spawn(async { handle_sway(window_event_sender, sway_socket) });
                } else {
                    error!("Unknown wayland compositor");
                }
            },
            Err(e) => {
                error!("{}", e);
            },
        }

        Self
    }
}

enum DisplayServer {
    X11,
    Wayland,
}

impl DisplayServer {
    fn detect() -> Result<Self> {
        match std::env::var("XDG_SESSION_TYPE") {
            Ok(ref session_type) if session_type == "wayland" => Ok(Self::Wayland),
            _ => Ok(Self::X11),
        }
    }
}

async fn handle_sway(window_event_sender: UnboundedSender<WindowEvent>, socket: impl AsRef<Path>) {
    let mut conn = tokio::net::UnixStream::connect(socket).await.unwrap();


    let payload = br#"["window"]"#;
    let payload_len: u32 = payload.len() as u32;
    let mut msg: Vec<u8> = vec![];

    msg.extend_from_slice(b"i3-ipc");
    msg.extend_from_slice(&payload_len.to_ne_bytes());
    msg.extend_from_slice(&2u32.to_ne_bytes());
    msg.extend_from_slice(payload);

    conn.write_all(&msg).await.unwrap();
    
    loop {
        let mut buf = bytes::BytesMut::new();
        conn.read_buf(&mut buf).await.unwrap();
        println!("{:?}", buf);
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
            if let Err(err) = handle_property_event(&conn, sender.clone(), event) {
                error!("error handling PropertyNotifyEvent: {err}");
            }
        }
    }
}

fn handle_property_event(
    conn: &RustConnection,
    sender: UnboundedSender<WindowEvent>,
    event: PropertyNotifyEvent,
) -> anyhow::Result<()> {
    let PropertyNotifyEvent { atom, state, .. } = event;
    if state == Property::NEW_VALUE {
        // TODO(mia): cache this
        let property_name = get_atom_name(conn, atom)?.reply()?.name;

        if property_name == b"_NET_ACTIVE_WINDOW" {
            trace!("active window changed");
            let focus = get_input_focus(conn)?.reply()?.focus;

            if let Ok(wm_class) = WmClass::get(conn, focus)?.reply() {
                let utf8 = String::from_utf8_lossy(wm_class.class());
                info!("focus changed to {utf8}");
            }

            let mut frame = focus;
            let query = query_tree(conn, focus)?.reply()?;
            let root = query.root;
            let mut parent = query.parent;

            while parent != root {
                frame = parent;
                parent = query_tree(conn, frame)?.reply()?.parent;
            }

            let geometry = get_geometry(conn, frame)?.reply()?;

            let _ = sender.send(WindowEvent::Reposition {
                x: geometry.x as i32,
                y: geometry.y as i32,
            });
        }
    }

    Ok(())
}
