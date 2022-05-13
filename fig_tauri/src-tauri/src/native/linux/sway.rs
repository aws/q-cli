use std::io::Cursor;
use std::path::Path;

use bytes::{
    Buf,
    BufMut,
    Bytes,
    BytesMut,
};
use serde_json::{
    json,
    Value,
};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::UnboundedSender;
use tracing::{
    debug,
    trace,
    warn,
};

use crate::utils::Rect;
use crate::window::WindowEvent;

struct I3Ipc {
    payload_type: u32,
    payload: Value,
}

impl I3Ipc {
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<(Self, usize), &'static str> {
        if src.remaining() < 6 {
            return Err("Header too short");
        }

        let mut magic_string = [0; 6];
        src.copy_to_slice(&mut magic_string);
        if &magic_string != b"i3-ipc" {
            return Err("Header is not `i3-ipc`");
        }

        if src.remaining() < 4 {
            return Err("Payload length too short");
        }

        let mut payload_length_buf = [0; 4];
        src.copy_to_slice(&mut payload_length_buf);
        let payload_length = u32::from_ne_bytes(payload_length_buf);

        if src.remaining() < 4 {
            return Err("Payload type too short");
        }

        let mut payload_type_buf = [0; 4];
        src.copy_to_slice(&mut payload_type_buf);
        let payload_type = u32::from_ne_bytes(payload_type_buf);

        if src.remaining() < payload_length as usize {
            return Err("Payload too short");
        }

        let mut payload_buf = vec![0; payload_length as usize];
        src.copy_to_slice(&mut payload_buf);

        let payload = serde_json::from_slice(&payload_buf).unwrap();

        Ok((Self { payload_type, payload }, 14 + payload_length as usize))
    }

    pub fn serialize(&self) -> Bytes {
        let payload = serde_json::to_vec(&self.payload).unwrap();

        let mut bytes = BytesMut::with_capacity(14);
        bytes.extend_from_slice(b"i3-ipc");
        bytes.put_u32_le(payload.len() as u32);
        bytes.put_u32_le(self.payload_type);
        bytes.extend_from_slice(&payload);

        bytes.freeze()
    }
}

pub async fn handle_sway(sender: UnboundedSender<WindowEvent>, socket: impl AsRef<Path>) {
    use tokio::io::AsyncReadExt;

    let mut conn = tokio::net::UnixStream::connect(socket).await.unwrap();

    let message = I3Ipc {
        payload_type: 2,
        payload: json!(["window"]),
    };

    conn.write_all(&message.serialize()).await.unwrap();

    let mut buf = bytes::BytesMut::new();
    loop {
        conn.read_buf(&mut buf).await.unwrap();
        match I3Ipc::parse(&mut Cursor::new(buf.as_ref())) {
            Ok((I3Ipc { payload, payload_type }, size)) => {
                trace!("{payload_type} {payload:?}");
                buf.advance(size);

                // Handle the message
                match payload_type {
                    0x80000003 => match payload.get("change") {
                        Some(Value::String(event)) if event == "focus" => {
                            if let Some(Value::Object(container)) = payload.get("container") {
                                let geometey = match container.get("geometry") {
                                    Some(Value::Object(geometry)) => {
                                        let x = geometry.get("x").and_then(|x| x.as_i64());
                                        let y = geometry.get("y").and_then(|y| y.as_i64());
                                        let width = geometry.get("width").and_then(|w| w.as_i64());
                                        let height = geometry.get("height").and_then(|h| h.as_i64());

                                        Some(Rect {
                                            x: x.unwrap_or(0),
                                            y: y.unwrap_or(0),
                                            width: width.unwrap_or(0),
                                            height: height.unwrap_or(0),
                                        })
                                    },
                                    _ => None,
                                };

                                let app_id = container.get("app_id").and_then(|x| x.as_str());

                                if let Some("org.kde.konsole") = app_id {
                                    sender.send(WindowEvent::Show).unwrap();
                                    sender
                                        .send(WindowEvent::Reposition {
                                            x: geometey.unwrap().x as i32,
                                            y: geometey.unwrap().y as i32,
                                        })
                                        .unwrap();
                                    sender.send(WindowEvent::Reanchor { x: 0, y: 0 }).unwrap();
                                }
                            }
                        },
                        Some(Value::String(event)) => warn!("Unknown event: {event}"),
                        event => warn!("Unknown event: {event:?}"),
                    },
                    _ => warn!("Unknown payload type: {payload_type}"),
                }
            },
            Err(_) => {},
        }
    }
}
