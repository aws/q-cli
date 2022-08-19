use std::sync::Arc;

use anyhow::Result;
use dbus::ibus_bus_new;
use hashbrown::HashSet;
use tracing::{
    debug,
    error,
};
use zbus::export::futures_util::TryStreamExt;
use zbus::fdo::DBusProxy;
use zbus::MessageStream;

use super::NativeState;
use crate::event::{
    Event,
    WindowEvent,
};
use crate::native::ActiveWindowData;
use crate::{
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

pub async fn init(proxy: EventLoopProxy, native_state: Arc<NativeState>) -> Result<()> {
    let ibus_connection = ibus_bus_new().await?;
    debug!("Connected to ibus");
    DBusProxy::new(&ibus_connection)
        .await?
        .add_match("eavesdrop=true")
        .await?;
    debug!("Added eavesdrop to ibus proxy");
    let mut stream = MessageStream::from(ibus_connection);
    tokio::spawn(async move {
        let mut active_input_contexts = HashSet::new();
        loop {
            match stream.try_next().await {
                Ok(Some(msg)) => {
                    if let (Some(member), Some(interface), Some(path)) = (msg.member(), msg.interface(), msg.path()) {
                        if interface.as_str() == "org.freedesktop.IBus.InputContext" {
                            match member.as_str() {
                                "FocusIn" => {
                                    debug!("FocusIn on {}", path.as_str());
                                    active_input_contexts.insert(path.as_str().to_owned());
                                },
                                "FocusOut" => {
                                    debug!("FocusOut on {}", path.as_str());
                                    active_input_contexts.remove(path.as_str());
                                },
                                "SetCursorLocation" => {
                                    if !active_input_contexts.contains(path.as_str()) {
                                        debug!("SetCursorLocation rejected on {}", path.as_str());
                                        continue;
                                    }
                                    let body: (i32, i32, i32, i32) = match msg.body() {
                                        Ok(body) => body,
                                        Err(err) => {
                                            error!(%err, "Failed deserializing message body");
                                            continue;
                                        },
                                    };
                                    if body == (0, 0, 0, 0) {
                                        debug!("null SetCursorLocation on {}", path.as_str());
                                    } else {
                                        debug!(
                                            "SetCursorLocation{{x: {}, y: {}}} on {}",
                                            body.0,
                                            body.1,
                                            path.as_str()
                                        );
                                        proxy
                                            .send_event(Event::WindowEvent {
                                                window_id: AUTOCOMPLETE_ID.clone(),
                                                window_event: WindowEvent::Reposition {
                                                    x: body.0,
                                                    y: body.1 + body.3,
                                                },
                                            })
                                            .unwrap();
                                    }
                                },
                                "SetCursorLocationRelative" => {
                                    if !active_input_contexts.contains(path.as_str()) {
                                        debug!("SetCursorLocationRelative rejected on {}", path.as_str());
                                        continue;
                                    }
                                    let body: (i32, i32, i32, i32) = match msg.body() {
                                        Ok(body) => body,
                                        Err(err) => {
                                            error!(%err, "Failed deserializing message body");
                                            continue;
                                        },
                                    };
                                    debug!(
                                        "SetCursorLocationRelative{{x: {}, y: {}}} on {}",
                                        body.0,
                                        body.1,
                                        path.as_str()
                                    );
                                    let abs: (i32, i32) = {
                                        let handle = native_state.active_window_data.lock();
                                        match *handle {
                                            Some(ActiveWindowData { x, y, off_x, off_y }) => {
                                                (body.0 + x + off_x, body.1 + y + off_y)
                                            },
                                            None => continue,
                                        }
                                    };
                                    debug!("resolved cursor to {{x: {}, y: {}}}", abs.0, abs.1,);
                                    proxy
                                        .send_event(Event::WindowEvent {
                                            window_id: AUTOCOMPLETE_ID.clone(),
                                            window_event: WindowEvent::Reposition {
                                                x: abs.0,
                                                y: abs.1 + body.3,
                                            },
                                        })
                                        .unwrap();
                                },
                                _ => {},
                            }
                        }
                    }
                },
                Ok(None) => {
                    debug!("Received end from ibus");
                    break;
                },
                Err(err) => {
                    error!(%err, "Failed receiving message from stream");
                },
            }
        }
    });

    Ok(())
}
