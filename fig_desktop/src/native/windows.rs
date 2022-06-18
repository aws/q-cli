use std::ffi::CStr;
use std::io::{
    stderr,
    stdin,
    stdout,
};
use std::sync::Arc;

use anyhow::Result;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use tracing::error;
use windows::Win32::Foundation::{
    BOOL,
    HWND,
    POINT,
};
use windows::Win32::Graphics::Gdi::ClientToScreen;
use windows::Win32::System::Com::{VARIANT, CoInitialize, CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::System::Console::{
    AttachConsole,
    FreeConsole,
    GetConsoleScreenBufferInfo,
    GetCurrentConsoleFont,
    GetStdHandle,
    ATTACH_PARENT_PROCESS,
    CONSOLE_FONT_INFO,
    CONSOLE_SCREEN_BUFFER_INFO,
    STD_OUTPUT_HANDLE,
};
use windows::Win32::System::ProcessStatus::{K32GetProcessImageFileNameA};
use windows::Win32::System::Threading::{GetCurrentThreadId, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
use windows::Win32::UI::Accessibility::{
    AccessibleObjectFromEvent,
    SetWinEventHook,
    UnhookWinEvent,
    HWINEVENTHOOK, AccessibleObjectFromWindow, CUIAutomation, IUIAutomation, TreeScope_Descendants, UIA_ControlTypePropertyId, UIA_TextControlTypeId,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetParent,
    GetWindowThreadProcessId,
    CHILDID_SELF,
    EVENT_OBJECT_LOCATIONCHANGE,
    EVENT_SYSTEM_FOREGROUND,
    OBJECT_IDENTIFIER,
    OBJID_WINDOW,
    WINEVENT_OUTOFCONTEXT,
    WINEVENT_SKIPOWNPROCESS, OBJID_CARET,
};
use windows::core::GUID;

use crate::event::{
    Event,
    NativeEvent,
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
        console_state: RwLock::new(ConsoleState::None),
    })
};

#[derive(Debug, Default)]
pub struct NativeState;

impl NativeState {
    pub fn handle(&self, event: NativeEvent) {
        match event {
            NativeEvent::EditBufferChanged => unsafe {
                update_caret_position();
            },
        }
    }
}

unsafe fn update_caret_position() {
    match *UNMANAGED.console_state.read() {
        ConsoleState::None => (),
        ConsoleState::Accessible { caret_position } => UNMANAGED.send_event(WindowEvent::Reposition {
            x: caret_position.x,
            y: caret_position.y,
        }),
        ConsoleState::Console { hwnd, process_id } => {
            let _lock1 = stderr().lock();
            let _lock2 = stdin().lock();
            let _lock3 = stdout().lock();

            FreeConsole();
            AttachConsole(process_id);
            let handle = GetStdHandle(STD_OUTPUT_HANDLE).unwrap();

            let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
            GetConsoleScreenBufferInfo(handle, &mut info);

            let mut font = CONSOLE_FONT_INFO::default();
            GetCurrentConsoleFont(handle, BOOL::from(false), &mut font);

            let mut position = POINT {
                x: ((info.dwCursorPosition.X - info.srWindow.Left) * font.dwFontSize.X) as i32,
                y: ((info.dwCursorPosition.Y - info.srWindow.Top) * font.dwFontSize.Y) as i32,
            };

            if ClientToScreen(hwnd, &mut position).as_bool() {
                UNMANAGED.send_event(WindowEvent::Reposition {
                    x: position.x,
                    y: position.y,
                });
            };

            FreeConsole();
            AttachConsole(ATTACH_PARENT_PROCESS);
        },
        ConsoleState::WindowsTerminal { hwnd, process_id } => {
            let _lock1 = stderr().lock();
            let _lock2 = stdin().lock();
            let _lock3 = stdout().lock();

            FreeConsole();
            AttachConsole(process_id);
            let handle = GetStdHandle(STD_OUTPUT_HANDLE).unwrap();

            let mut info = CONSOLE_SCREEN_BUFFER_INFO::default();
            GetConsoleScreenBufferInfo(handle, &mut info);

            let mut font = CONSOLE_FONT_INFO::default();
            GetCurrentConsoleFont(handle, BOOL::from(false), &mut font);

            let mut position = POINT {
                x: ((info.dwCursorPosition.X - info.srWindow.Left) * font.dwFontSize.X) as i32,
                y: ((info.dwCursorPosition.Y - info.srWindow.Top) * font.dwFontSize.Y) as i32,
            };

            if ClientToScreen(hwnd, &mut position).as_bool() {
                UNMANAGED.send_event(WindowEvent::Reposition {
                    x: position.x,
                    y: position.y,
                });
            };

            FreeConsole();
            AttachConsole(ATTACH_PARENT_PROCESS);
        }
    }
}

enum ConsoleState {
    None,
    Accessible { caret_position: POINT },
    Console { hwnd: HWND, process_id: u32 },
    WindowsTerminal { hwnd: HWND, process_id: u32 },
}

#[allow(dead_code)]
struct Unmanaged {
    main_thread: u32,
    previous_focus: RwLock<Option<u32>>,
    event_sender: RwLock<Option<EventLoopProxy>>,
    foreground_hook: RwLock<HWINEVENTHOOK>,
    location_hook: RwLock<Option<HWINEVENTHOOK>>,
    console_state: RwLock<ConsoleState>,
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

    #[allow(unused_variables)]
    pub fn lookup(name: &str) -> Option<ProcessedAsset> {
        None
    }
}

#[allow(unused_variables)]
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
        e if e == EVENT_SYSTEM_FOREGROUND
            && OBJECT_IDENTIFIER(id_object) == OBJID_WINDOW
            && id_child == CHILDID_SELF as i32 =>
        {
            if let Some(hook) = UNMANAGED.location_hook.write().take() {
                UnhookWinEvent(hook);
            }

            UNMANAGED.send_event(WindowEvent::Hide);

            let mut process_id: u32 = 0;
            let thread_id = GetWindowThreadProcessId(hwnd, &mut process_id);
            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id).unwrap();
            let mut process_name = vec![0; 256];
            let len = K32GetProcessImageFileNameA(process_handle, &mut process_name) as usize;
            process_name.truncate(len + 1);
            let title = match CStr::from_bytes_with_nul(&process_name).expect("Missing null terminator").to_str() {
                Ok(process_name) => match process_name.split('\\').last() {
                    Some(title) => match title.strip_suffix(".exe") {
                        Some(title) => title,
                        None => return,
                    },
                    None => return,
                },
                Err(_) => return,
            };

            println!("{}", title);

            match title {
                title if ["Hyper", "Code"].contains(&title) => (),
                title if ["cmd", "powershell"].contains(&title) => {
                    let hwnd = GetParent(hwnd);
                    let mut process_id: u32 = 0;
                    GetWindowThreadProcessId(hwnd, &mut process_id);
                    *UNMANAGED.console_state.write() = ConsoleState::Console { hwnd, process_id }
                },
                title if title == "WindowsTerminal" => {
                    error!("We don't support this yet");
                    //CoInitialize(std::ptr::null_mut()).unwrap();
                    //let automation: IUIAutomation = CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).unwrap();
                    //let window = automation.ElementFromHandle(hwnd).unwrap();
//
                    //let interest = automation.CreatePropertyCondition(UIA_ControlTypePropertyId, VARIANT::from(UIA_TextControlTypeId)).unwrap();
//
                    //if let Ok(terminal) = window.FindFirst(TreeScope_Descendants, &interest) {
                    //    
                    //}
                    //
                    //let mut process_id: u32 = 0;
                    //GetWindowThreadProcessId(hwnd, &mut process_id);
                    //*UNMANAGED.console_state.write() = ConsoleState::WindowsTerminal { hwnd, process_id }
                },
                _ => {
                    *UNMANAGED.console_state.write() = ConsoleState::None;
                    return;
                },
            }

            UNMANAGED.location_hook.write().replace(SetWinEventHook(
                EVENT_OBJECT_LOCATIONCHANGE,
                EVENT_OBJECT_LOCATIONCHANGE,
                None,
                Some(win_event_proc),
                process_id,
                thread_id,
                WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
            ));
        },
        e if e == EVENT_OBJECT_LOCATIONCHANGE
            && OBJECT_IDENTIFIER(id_object) == OBJID_WINDOW
            && id_child == CHILDID_SELF as i32 =>
        {
            UNMANAGED.send_event(WindowEvent::Hide)
        },
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
                        *UNMANAGED.console_state.write() = ConsoleState::Accessible {
                            caret_position: POINT { x: left, y: top },
                        }
                    }
                }
            }
        },
        _ => (),
    }
}
