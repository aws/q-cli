use std::sync::Arc;

use anyhow::Result;
use dbus::ibus_bus_new;
use fig_util::terminal::PositioningKind;
use hashbrown::HashSet;
use tracing::{
    debug,
    error,
};
use wry::application::dpi::{
    LogicalPosition,
    LogicalSize,
    PhysicalPosition,
    Position,
};
use zbus::export::futures_util::TryStreamExt;
use zbus::fdo::DBusProxy;
use zbus::MessageStream;

use super::PlatformStateImpl;
use crate::event::{
    Event,
    WindowEvent,
    WindowPosition,
};
use crate::platform::ActiveWindowData;
use crate::{
    EventLoopProxy,
    AUTOCOMPLETE_ID,
};

pub(super) async fn init(proxy: EventLoopProxy, platform_state: Arc<PlatformStateImpl>) -> Result<()> {
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
                                        let positioning_kind = platform_state
                                            .active_terminal
                                            .lock()
                                            .as_ref()
                                            .map(|x| x.positioning_kind())
                                            .unwrap_or(PositioningKind::Physical);
                                        proxy
                                            .send_event(Event::WindowEvent {
                                                window_id: AUTOCOMPLETE_ID.clone(),
                                                window_event: WindowEvent::UpdateWindowGeometry {
                                                    position: Some(WindowPosition::Absolute(position(
                                                        positioning_kind,
                                                        body.0,
                                                        body.1 + body.3,
                                                    ))),
                                                    size: None,
                                                    anchor: None,
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
                                        "SetCursorLocationRelative{{x: {}, y: {}, h: {}}} on {}",
                                        body.0,
                                        body.1,
                                        body.3,
                                        path.as_str()
                                    );
                                    let abs: (i32, i32) = {
                                        let handle = platform_state.active_window_data.lock();
                                        match *handle {
                                            Some(ActiveWindowData {
                                                outer_x,
                                                outer_y,
                                                scale,
                                                ..
                                            }) => (
                                                (body.0 as f32 / scale).round() as i32 + outer_x,
                                                (body.1 as f32 / scale).round() as i32 + outer_y
                                                    - (body.3 as f32 / scale).round() as i32,
                                            ),
                                            None => continue,
                                        }
                                    };
                                    debug!("resolved cursor to {{x: {}, y: {}}}", abs.0, abs.1,);
                                    proxy
                                        .send_event(Event::WindowEvent {
                                            window_id: AUTOCOMPLETE_ID.clone(),
                                            window_event: WindowEvent::UpdateWindowGeometry {
                                                position: Some(WindowPosition::RelativeToCaret {
                                                    caret_position: LogicalPosition {
                                                        x: abs.0 as f64,
                                                        y: abs.1 as f64,
                                                    },
                                                    caret_size: LogicalSize {
                                                        width: body.2 as f64,
                                                        height: body.3 as f64,
                                                    },
                                                }),
                                                size: None,
                                                anchor: None,
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
                    break;
                },
            }
        }
    });

    Ok(())
}

fn position(kind: PositioningKind, x: i32, y: i32) -> Position {
    match kind {
        PositioningKind::Logical => Position::Logical(LogicalPosition {
            x: x as f64,
            y: y as f64,
        }),
        PositioningKind::Physical => Position::Physical(PhysicalPosition { x, y }),
    }
}
