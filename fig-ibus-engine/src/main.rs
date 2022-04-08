use glib::{g_log, LogLevel, StaticType};
use ibus::traits::{BusExt, FactoryExt};

mod imp {
    use glib::subclass::prelude::*;
    use glib::{g_log, LogLevel};
    use ibus::traits::EngineExt;
    use parking_lot::Mutex;
    use std::os::unix::net::UnixStream;
    use std::time::{Duration, Instant};

    pub struct FigIBusEngine {
        cursor_position: Mutex<(i32, i32, i32, i32)>,
        socket_connection: Mutex<Option<Result<UnixStream, Instant>>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for FigIBusEngine {
        const NAME: &'static str = "FigIBusEngine";
        type Type = super::FigIBusEngine;
        type ParentType = ibus::Engine;

        fn new() -> Self {
            Self {
                cursor_position: Mutex::default(),
                socket_connection: Mutex::new(None),
            }
        }
    }

    impl ObjectImpl for FigIBusEngine {
        fn constructed(&self, obj: &Self::Type) {
            self.parent_constructed(obj);
            g_log!("Fig", LogLevel::Info, "Engine constructed");
            obj.connect_set_cursor_location(|engine, x, y, w, h| {
                *engine.imp().cursor_position.lock() = (x, y, w, h);
            });
            obj.connect_process_key_event(|engine, _, _, _| {
                use fig_proto::linux::*;
                let imp = engine.imp();
                let cursor_position = *imp.cursor_position.lock();
                send_command(
                    imp,
                    AppCommand {
                        command: Some(app_command::Command::SetCursorPosition(
                            SetCursorPositionCommand {
                                x: cursor_position.0,
                                y: cursor_position.1,
                                width: cursor_position.2,
                                height: cursor_position.3,
                            },
                        )),
                    },
                );

                false
            });
            obj.connect_focus_in(handle_focus_change);
            obj.connect_focus_out(handle_focus_change);
        }
    }

    fn handle_focus_change(engine: &super::FigIBusEngine) {
        use fig_proto::linux::*;
        send_command(
            engine.imp(),
            AppCommand {
                command: Some(app_command::Command::FocusChangeCommand(Empty {})),
            },
        );
    }

    fn send_command(imp: &FigIBusEngine, command: fig_proto::linux::AppCommand) {
        if let Some(mut handle) = imp.socket_connection.try_lock() {
            if match &*handle {
                Some(Err(time)) => {
                    if Instant::now() - *time > Duration::new(10, 0) {
                        *handle = Some(get_stream());
                        true
                    } else {
                        false
                    }
                }
                None => {
                    *handle = Some(get_stream());
                    true
                }
                _ => true,
            } {}

            if let Some(Ok(stream)) = &mut *handle {
                if let Err(err) = fig_ipc::send_message_sync(stream, command) {
                    g_log!("Fig", LogLevel::Error, "Failed sending message: {:?}", err);
                }
            }
        };
    }

    fn get_stream() -> Result<UnixStream, Instant> {
        fig_ipc::connect_sync(fig_ipc::get_fig_linux_socket_path()).map_err(|err| {
            g_log!(
                "Fig",
                LogLevel::Error,
                "Failed connecting to socket: {:?}",
                err
            );
            Instant::now()
        })
    }

    unsafe impl IsSubclassable<FigIBusEngine> for ibus::Engine {}
}

glib::wrapper! {
    pub struct FigIBusEngine(ObjectSubclass<imp::FigIBusEngine>)
        @extends ibus::Engine;
}

fn main() {
    let bus = ibus::Bus::new();

    let factory = ibus::Factory::new(&bus.connection().unwrap());
    factory.add_engine("FigIBusEngine", FigIBusEngine::static_type());

    let component = ibus::Component::from_file("engine.xml");
    bus.register_component(&component);
    // bus.request_name("org.freedesktop.IBus.FigIBusEngine", 0);
    g_log!("Fig", LogLevel::Info, "Engine registered");

    glib::MainLoop::new(None, false).run();
}
