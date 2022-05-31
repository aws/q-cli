use std::io::Cursor;
use std::path::Path;

use anyhow::{
    anyhow,
    Error,
};
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
use tracing::{
    error,
    info,
    trace,
    warn,
};
use wry::application::event_loop::EventLoopProxy;

use crate::utils::Rect;
use crate::window::FigWindowEvent;
use crate::{
    FigEvent,
    AUTOCOMPLETE_ID,
};

struct I3Ipc {
    payload_type: u32,
    payload: Value,
}

enum ParseResult {
    Ok { i3ipc: I3Ipc, size: usize },
    Incomplete,
    Error(Error),
}

impl I3Ipc {
    pub fn parse(src: &mut Cursor<&[u8]>) -> ParseResult {
        if src.remaining() < 14 {
            return ParseResult::Incomplete;
        }

        let mut magic_string = [0; 6];
        src.copy_to_slice(&mut magic_string);
        if &magic_string != b"i3-ipc" {
            return ParseResult::Error(anyhow!("header is not `i3-ipc`"));
        }

        let mut payload_length_buf = [0; 4];
        src.copy_to_slice(&mut payload_length_buf);
        let payload_length = u32::from_ne_bytes(payload_length_buf);

        let mut payload_type_buf = [0; 4];
        src.copy_to_slice(&mut payload_type_buf);
        let payload_type = u32::from_ne_bytes(payload_type_buf);

        if src.remaining() < payload_length as usize {
            return ParseResult::Incomplete;
        }

        let mut payload_buf = vec![0; payload_length as usize];
        src.copy_to_slice(&mut payload_buf);

        let payload = match serde_json::from_slice(&payload_buf) {
            Ok(payload) => payload,
            Err(err) => return ParseResult::Error(err.into()),
        };

        ParseResult::Ok {
            i3ipc: Self { payload_type, payload },
            size: 14 + payload_length as usize,
        }
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

pub async fn handle_sway(proxy: EventLoopProxy<FigEvent>, socket: impl AsRef<Path>) {
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
            ParseResult::Ok {
                i3ipc: I3Ipc { payload, payload_type },
                size,
            } => {
                trace!("{payload_type} {payload:?}");
                buf.advance(size);

                // Handle the message
                match payload_type {
                    2 => {
                        if let Some(Value::Bool(true)) = payload.get("success") {
                            info!("Successfuly subscribed to sway events");
                        } else {
                            warn!("Failed to subscribe to sway events: {payload}");
                        }
                    },
                    0x80000003 => match payload.get("change") {
                        Some(Value::String(event)) if event == "focus" => {
                            if let Some(Value::Object(container)) = payload.get("container") {
                                let _geometey = match container.get("geometry") {
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
                                    proxy
                                        .send_event(FigEvent::WindowEvent {
                                            fig_id: AUTOCOMPLETE_ID.clone(),
                                            window_event: FigWindowEvent::Show,
                                        })
                                        .unwrap();
                                    // proxy
                                    //    .send_event(FigEvent::WindowEvent {
                                    //        fig_id: AUTOCOMPLETE_ID.clone(),
                                    //        window_event: FigWindowEvent::Reposition {
                                    //            x: geometey.unwrap().x as i32,
                                    //            y: geometey.unwrap().y as i32,
                                    //        },
                                    //    })
                                    //    .unwrap();
                                    proxy
                                        .send_event(FigEvent::WindowEvent {
                                            fig_id: AUTOCOMPLETE_ID.clone(),
                                            window_event: FigWindowEvent::Reanchor { x: 0, y: 0 },
                                        })
                                        .unwrap();
                                }
                            }
                        },
                        Some(Value::String(event)) => trace!("Unknown event: {event}"),
                        event => trace!("Unknown event: {event:?}"),
                    },
                    _ => trace!("Unknown payload type: {payload_type} {payload:?}"),
                }
            },
            ParseResult::Incomplete => continue,
            ParseResult::Error(error) => {
                error!("Failed to parse sway message: {error}");
                break;
            },
        }
    }
}
