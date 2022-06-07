use std::ffi::CStr;
use std::sync::Arc;

use anyhow::Result;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use windows::Win32::Foundation::{
    HWND,
    POINT,
    RECT,
};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::Com::VARIANT;
use windows::Win32::System::Console::{
    AttachConsole,
    FreeConsole,
};
use windows::Win32::System::Threading::{
    AttachThreadInput,
    GetCurrentThreadId,
};
use windows::Win32::UI::Accessibility::{
    AccessibleObjectFromEvent,
    SetWinEventHook,
    UnhookWinEvent,
    HWINEVENTHOOK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCaretPos,
    GetWindowRect,
    GetWindowTextA,
    GetWindowThreadProcessId,
    CHILDID_SELF,
    EVENT_CONSOLE_CARET,
    EVENT_OBJECT_LOCATIONCHANGE,
    EVENT_SYSTEM_FOREGROUND,
    OBJECT_IDENTIFIER,
    OBJID_CARET,
    OBJID_QUERYCLASSNAMEIDX,
    OBJID_WINDOW,
    WINEVENT_OUTOFCONTEXT,
    WINEVENT_SKIPOWNPROCESS,
};

use crate::event::{
    Event,
    WindowEvent,
};
use crate::window::CursorPositionKind;
use crate::{
    EventLoopProxy,
    GlobalState,
    AUTOCOMPLETE_ID,
};

pub const SHELL: &str = "wsl";
pub const SHELL_ARGS: [&str; 0] = [];
pub const CURSOR_POSITION_KIND: CursorPositionKind = CursorPositionKind::Relative;

static UNMANAGED: Lazy<Unmanaged> = unsafe {
    Lazy::new(|| Unmanaged {
        main_thread: GetCurrentThreadId(),
        previous_focus: RwLock::new(None),
        event_sender: RwLock::new(Option::<EventLoopProxy>::None),
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

#[derive(Debug, Default)]
pub struct NativeState;

#[allow(dead_code)]
struct Unmanaged {
    main_thread: u32,
    previous_focus: RwLock<Option<u32>>,
    event_sender: RwLock<Option<EventLoopProxy>>,
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
            .send_event(Event::WindowEvent {
                window_id: AUTOCOMPLETE_ID,
                window_event: event,
            })
            .expect("Failed to emit window event");
    }
}

pub mod icons {
    use crate::icons::ProcessedAsset;

    pub fn lookup(name: &str) -> Option<ProcessedAsset> {
        None
    }
}

pub async fn init(global_state: Arc<GlobalState>, proxy: EventLoopProxy) -> Result<()> {
    UNMANAGED.event_sender.write().replace(proxy);

    Ok(())
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
    match event {
        // The focused app has been changed
        e if e == EVENT_SYSTEM_FOREGROUND => {
            if OBJECT_IDENTIFIER(id_object) != OBJID_WINDOW && id_child != CHILDID_SELF as i32 {
                return;
            }

            if let Some(hook) = UNMANAGED.location_hook.write().take() {
                UnhookWinEvent(hook);
            }

            // SAFETY: hwnd must be valid and process_id must point to allocated memory
            let mut process_id: u32 = 0;
            let thread_id = GetWindowThreadProcessId(hwnd, &mut process_id);
            FreeConsole();
            match AttachConsole(process_id).as_bool() {
                true => {
                    UNMANAGED.location_hook.write().replace(SetWinEventHook(
                        EVENT_CONSOLE_CARET,
                        EVENT_CONSOLE_CARET,
                        None,
                        Some(win_event_proc),
                        process_id,
                        thread_id,
                        WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                    ));

                    UNMANAGED.send_event(WindowEvent::Show);
                },
                false => {
                    // let mut class_name = vec![0; 256];
                    // let len = GetWindowTextA(hwnd, &mut class_name) as usize;
                    // class_name.truncate(len + 1);
                    // let title = match CStr::from_bytes_with_nul(&class_name)
                    //    .expect("Missing null terminator")
                    //    .to_str()
                    //{
                    //    Ok(title) => title,
                    //    // Window title is non-utf8, shouldn't be a terminal we care about
                    //    Err(_) => return,
                    //};

                    // if title == "Hyper" {
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
                    //} else {
                    //    UNMANAGED.send_event(WindowEvent::Hide);
                    //}
                },
            }
        },
        // The focused app has moved
        e if e == EVENT_OBJECT_LOCATIONCHANGE && OBJECT_IDENTIFIER(id_object) == OBJID_CARET => {
            let mut acc = None;
            let mut varchild = VARIANT::default();
            if AccessibleObjectFromEvent(hwnd, id_object as u32, id_child as u32, &mut acc, &mut varchild).is_ok() {
                if let Some(acc) = acc {
                    let mut left = 0;
                    let mut top = 0;
                    let mut width = 0;
                    let mut height = 0;
                    if acc
                        .accLocation(&mut left, &mut top, &mut width, &mut height, varchild)
                        .is_ok()
                    {
                        UNMANAGED.send_event(WindowEvent::Reposition { x: left, y: top });
                    }
                }
            }
        },
        _ => (),
    }
}
