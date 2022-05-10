use std::borrow::Cow;
use std::io::Cursor;
use std::mem::size_of;
use std::path::Path;

use anyhow::Result;
use bytes::Buf;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{
    debug,
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
                    tauri::async_runtime::spawn(async { handle_sway(window_event_sender, sway_socket).await });
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

struct Container {
    app_id: String,
}

struct Window {
    change: String,
    container: Container,
}

struct I3Ipc {
    payload_type: u32,
    payload: serde_json::Value,
}

impl I3Ipc {
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<(Self, usize), &'static str> {
        use std::io::Read;

        if src.remaining() < 6 {
            return Err("Header too short");
        }

        let mut magic_string = [0; 6];

        src.read_exact(&mut magic_string);

        if &magic_string != b"i3-ipc" {
            return Err("Header is not `i3-ipc`");
        }

        if src.remaining() < size_of::<u32>() {
            return Err("Payload length too short");
        }

        let mut payload_length_buf = [0; size_of::<u32>()];
        src.read_exact(&mut payload_length_buf);
        let payload_length = u32::from_ne_bytes(payload_length_buf);

        if src.remaining() < size_of::<u32>() {
            return Err("Payload type too short");
        }

        let mut payload_type_buf = [0; size_of::<u32>()];
        src.read_exact(&mut payload_type_buf);
        let payload_type = u32::from_ne_bytes(payload_type_buf);

        if src.remaining() < payload_length as usize {
            return Err("Payload too short");
        }

        let mut payload_buf = vec![0; payload_length as usize];
        src.read_exact(&mut payload_buf);

        let payload = serde_json::from_slice(&payload_buf).unwrap();

        Ok((
            Self { payload_type, payload },
            6 + size_of::<u32>() * 2 + payload_length as usize,
        ))
    }
}

async fn handle_sway(window_event_sender: UnboundedSender<WindowEvent>, socket: impl AsRef<Path>) {
    use tokio::io::AsyncReadExt;

    let mut conn = tokio::net::UnixStream::connect(socket).await.unwrap();
    let mut msg = bytes::BytesMut::new();

    let payload = br#"["window"]"#;
    let payload_len: u32 = payload.len() as u32;

    msg.extend_from_slice(b"i3-ipc");
    msg.extend_from_slice(&payload_len.to_ne_bytes());
    msg.extend_from_slice(&2u32.to_ne_bytes());
    msg.extend_from_slice(payload);

    conn.write_all(&msg).await.unwrap();

    let mut buf = bytes::BytesMut::new();
    loop {
        conn.read_buf(&mut buf).await.unwrap();
        match I3Ipc::parse(&mut Cursor::new(buf.as_ref())) {
            Ok((ipc, size)) => {
                buf.advance(size);
                println!("{:?}", ipc.payload);
            },
            Err(err) => {},
        }
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
