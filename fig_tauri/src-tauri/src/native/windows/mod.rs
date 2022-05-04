mod ipc;

use std::ffi::CStr;
use std::path::Path;

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use tokio::sync::mpsc::UnboundedSender;
use windows::Win32::{
    Foundation::HWND,
    Foundation::RECT,
    System::Com,
    UI::{
        Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
        WindowsAndMessaging::{
            GetWindowRect, GetWindowTextA, GetWindowThreadProcessId, CHILDID_SELF,
            EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND, EVENT_UIA_PROPID_END,
            EVENT_UIA_PROPID_START, OBJECT_IDENTIFIER, OBJID_QUERYCLASSNAMEIDX, OBJID_WINDOW,
            WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
        },
    },
};

use crate::window::WindowEvent;

pub const SHELL: &str = "wsl";
pub const SHELL_ARGS: [&str; 0] = [];

static UNMANAGED: Lazy<Unmanaged> = unsafe {
    Lazy::new(|| Unmanaged {
        event_sender: RwLock::new(None),
        foreground_hook: RwLock::new(SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(win_event_proc),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        )),
        location_hook: RwLock::new(None),
        caret_hook: RwLock::new(None),
    })
};

#[derive(Debug)]
pub struct NativeState;

impl NativeState {
    pub fn new(window_event_sender: UnboundedSender<WindowEvent>) -> Self {
        UNMANAGED.event_sender.write().replace(window_event_sender);
        NativeState
    }
}

#[allow(dead_code)]
struct Unmanaged {
    event_sender: RwLock<Option<UnboundedSender<WindowEvent>>>,
    foreground_hook: RwLock<HWINEVENTHOOK>,
    location_hook: RwLock<Option<HWINEVENTHOOK>>,
    caret_hook: RwLock<Option<HWINEVENTHOOK>>,
}

impl Unmanaged {
    pub fn send_event(&self, event: WindowEvent) {
        self.event_sender
            .read()
            .clone()
            .expect("Window event sender was none")
            .send(event)
            .expect("Failed to emit window event");
    }
}

pub struct Listener(ipc::WindowsListener);

impl Listener {
    pub fn bind(path: &Path) -> Self {
        Self(ipc::WindowsListener::bind(path).expect("Failed to bind to socket"))
    }

    pub async fn accept(&self) -> Result<ipc::WindowsStream, ipc::WinSockError> {
        self.0.accept().await
    }
}

unsafe extern "system" fn win_event_proc(
    _hook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _id_event_thread: u32,
    _time: u32,
) {
    if id_child != CHILDID_SELF as i32 {
        return;
    }

    match event {
        // The focused app has been changed
        e if e == EVENT_SYSTEM_FOREGROUND => {
            if OBJECT_IDENTIFIER(id_object) != OBJID_WINDOW {
                return;
            }

            if let Some(hook) = UNMANAGED.location_hook.write().take() {
                UnhookWinEvent(hook);
            }

            let mut class_name = vec![0; 256];
            let len = GetWindowTextA(hwnd, &mut class_name) as usize;
            class_name.truncate(len + 1);
            let title = match CStr::from_bytes_with_nul(&class_name)
                .expect("Missing null terminator")
                .to_str()
            {
                Ok(title) => title,
                // Window title is non-utf8, shouldn't be a terminal we care about
                Err(_) => return,
            };

            if title == "Hyper" {
                // SAFETY: hwnd must be valid and process_id must point to allocated memory
                let mut process_id: u32 = 0;
                let thread_id = GetWindowThreadProcessId(hwnd, &mut process_id);

                // SAFETY:
                // - eventmin and eventmax must be a valid range
                // - hmodwineventproc must be null when `WINEVENT_OUTOFCONTEXT` is specified
                // - pfnwineventproc must be a valid WINEVENTPROC function
                // - idprocess and idthread must be valid or 0
                UNMANAGED.location_hook.write().replace(SetWinEventHook(
                    EVENT_OBJECT_LOCATIONCHANGE,
                    EVENT_OBJECT_LOCATIONCHANGE,
                    None,
                    Some(win_event_proc),
                    process_id,
                    thread_id,
                    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                ));

                UNMANAGED.send_event(WindowEvent::Show);
            } else {
                UNMANAGED.send_event(WindowEvent::Hide);
            }
        }
        // The focused app has moved
        e if e == EVENT_OBJECT_LOCATIONCHANGE => {
            let mut rect = RECT::default();
            if OBJECT_IDENTIFIER(id_object) == OBJID_WINDOW {
                if GetWindowRect(hwnd, &mut rect).as_bool() {
                    UNMANAGED.send_event(WindowEvent::Reposition {
                        x: rect.left,
                        y: rect.bottom,
                    });
                }
            } else if OBJECT_IDENTIFIER(id_object) == OBJID_QUERYCLASSNAMEIDX {
                todo!();
            }
        }
        _ => (),
    }
}
