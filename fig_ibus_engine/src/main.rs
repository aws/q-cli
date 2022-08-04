use std::ffi::CStr;
use std::os::raw::c_char;
use std::os::unix::net::UnixStream;
use std::time::{
    Duration,
    Instant,
};

use fig_util::directories;
use parking_lot::Mutex;
use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};

static SOCKET_CONNECTION: Mutex<Option<Result<UnixStream, Instant>>> = parking_lot::const_mutex(None);

fn send_hook(hook: fig_proto::local::Hook) {
    use fig_proto::local::*;
    if let Some(mut handle) = SOCKET_CONNECTION.try_lock() {
        if match &*handle {
            Some(Err(time)) => {
                if time.elapsed() > Duration::new(5, 0) {
                    *handle = Some(get_stream());
                    true
                } else {
                    false
                }
            },
            None => {
                *handle = Some(get_stream());
                true
            },
            _ => true,
        } {}

        if let Some(Ok(stream)) = &mut *handle {
            if let Err(err) = fig_ipc::send_message_sync(stream, LocalMessage {
                r#type: Some(local_message::Type::Hook(hook)),
            }) {
                *handle = None;
                warn!("Failed sending message: {err:?}");
            }
        }
    };
}

fn get_stream() -> Result<UnixStream, Instant> {
    fig_ipc::connect_sync(directories::fig_socket_path().expect("Failed getting fig socket")).map_err(|err| {
        warn!("Failed connecting to socket: {err:?}");
        Instant::now()
    })
}

extern "C" {
    fn fig_engine_main(
        started_by_ibus: bool,
        cursor_callback: extern "C" fn(i32, i32, i32, i32),
        log_callback: extern "C" fn(u8, *const c_char),
    );
}

extern "C" fn cursor_callback(x: i32, y: i32, w: i32, h: i32) {
    use fig_proto::local::*;
    send_hook(Hook {
        hook: Some(hook::Hook::CursorPosition(CursorPositionHook {
            x,
            y,
            width: w,
            height: h,
        })),
    });
}

extern "C" fn log_warning(level: u8, message: *const c_char) {
    // SAFETY: All the messages we recieve can be seen in `engine.vala`. They do not contain invalid
    // characters and they properly end with a null byte (vala upholds this).
    let message = unsafe { CStr::from_ptr(message as *mut c_char) };
    if let Ok(message) = message.to_str() {
        match level {
            0 => trace!("{message}"),
            1 => debug!("{message}"),
            2 => info!("{message}"),
            3 => warn!("{message}"),
            4 => error!("{message}"),
            _ => panic!("invalid log level: {level}"),
        };
    }
}

fn main() {
    let _guard = fig_log::Logger::new()
        .with_file("ibus_engine.log")
        .with_stdout()
        .init()
        .expect("Failed initializing logger");
    unsafe {
        fig_engine_main(std::env::args().any(|x| x == "ibus"), cursor_callback, log_warning);
    }
}
