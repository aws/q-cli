use std::ffi::CString;
use std::os::raw::c_char;
use std::os::unix::net::UnixStream;
use std::time::{
    Duration,
    Instant,
};

use parking_lot::lock_api::RawMutex as _;
use parking_lot::{
    Mutex,
    RawMutex,
};

static SOCKET_CONNECTION: Mutex<Option<Result<UnixStream, Instant>>> = Mutex::const_new(RawMutex::INIT, None);

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
                log_warning(format!("Failed sending message: {err:?}"));
            }
        }
    };
}

fn get_stream() -> Result<UnixStream, Instant> {
    fig_ipc::connect_sync(fig_ipc::get_fig_socket_path()).map_err(|err| {
        log_warning(format!("Failed connecting to socket: {err:?}"));
        Instant::now()
    })
}

extern "C" {
    fn fig_engine_main(started_by_ibus: bool, cursor_callback: extern "C" fn(i32, i32, i32, i32));

    fn fig_log_warning(message: *const c_char);
}

fn log_warning(message: String) {
    let cstring = CString::new(message).unwrap();
    unsafe {
        fig_log_warning(cstring.as_ptr());
    }
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

fn main() {
    unsafe {
        fig_engine_main(std::env::args().any(|x| x == "ibus"), cursor_callback);
    }
}
