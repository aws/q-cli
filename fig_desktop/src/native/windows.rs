use std::ffi::CStr;
use std::mem::ManuallyDrop;
use std::sync::Arc;

use anyhow::{
    anyhow,
    Result,
};
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use tracing::debug;
use windows::Win32::Foundation::{
    HWND,
    RECT,
};
use windows::Win32::System::Com::{
    CoCreateInstance,
    CoInitialize,
    CLSCTX_INPROC_SERVER,
    VARIANT,
    VARIANT_0,
    VARIANT_0_0,
    VARIANT_0_0_0,
};
use windows::Win32::System::Ole::VT_BOOL;
use windows::Win32::System::ProcessStatus::K32GetProcessImageFileNameA;
use windows::Win32::System::Threading::{
    GetCurrentThreadId,
    OpenProcess,
    PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::Accessibility::{
    AccessibleObjectFromEvent,
    CUIAutomation,
    IUIAutomation,
    IUIAutomationTextPattern,
    SetWinEventHook,
    TextUnit_Character,
    TreeScope_Descendants,
    UIA_HasKeyboardFocusPropertyId,
    UIA_IsTextPatternAvailablePropertyId,
    UIA_TextPatternId,
    UnhookWinEvent,
    HWINEVENTHOOK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow,
    GetWindowThreadProcessId,
    CHILDID_SELF,
    EVENT_OBJECT_LOCATIONCHANGE,
    EVENT_SYSTEM_FOREGROUND,
    OBJECT_IDENTIFIER,
    OBJID_CARET,
    OBJID_WINDOW,
    WINEVENT_OUTOFCONTEXT,
    WINEVENT_SKIPOWNPROCESS,
};

use crate::event::{
    Event,
    NativeEvent,
    WindowEvent,
};
use crate::{
    EventLoopProxy,
    GlobalState,
    AUTOCOMPLETE_ID,
};

pub const SHELL: &str = "bash";

#[repr(C)]
struct AutomationTable(IUIAutomation);

unsafe impl Sync for AutomationTable {}
unsafe impl Send for AutomationTable {}

static UNMANAGED: Lazy<Unmanaged> = unsafe {
    Lazy::new(|| Unmanaged {
        main_thread: GetCurrentThreadId(),
        previous_focus: RwLock::new(None),
        global_state: RwLock::new(Option::<Arc<GlobalState>>::None),
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
        automation_instance: AutomationTable({
            CoInitialize(std::ptr::null_mut()).unwrap();
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).unwrap()
        }),
    })
};

const VT_TRUE: VARIANT = VARIANT {
    Anonymous: VARIANT_0 {
        Anonymous: ManuallyDrop::new(VARIANT_0_0 {
            vt: VT_BOOL.0 as u16,
            wReserved1: 0,
            wReserved2: 0,
            wReserved3: 0,
            Anonymous: VARIANT_0_0_0 {
                boolVal: unsafe { std::mem::transmute(0xffff_u16) },
            },
        }),
    },
};

#[derive(Debug, Default)]
pub struct NativeState;

impl NativeState {
    pub fn handle(&self, event: NativeEvent) -> Result<()> {
        match event {
            NativeEvent::EditBufferChanged => unsafe {
                let console_state = *UNMANAGED.console_state.read();
                match console_state {
                    ConsoleState::None => (),
                    ConsoleState::Console { hwnd } => {
                        let automation = &UNMANAGED.automation_instance.0;
                        let window = automation.ElementFromHandle(hwnd)?;

                        let interest = automation.CreateAndCondition(
                            &automation.CreatePropertyCondition(UIA_HasKeyboardFocusPropertyId, &VT_TRUE)?,
                            &automation.CreatePropertyCondition(UIA_IsTextPatternAvailablePropertyId, &VT_TRUE)?,
                        )?;

                        let inner = window.FindFirst(TreeScope_Descendants, &interest)?;
                        let text_pattern = inner.GetCurrentPatternAs::<IUIAutomationTextPattern>(UIA_TextPatternId)?;
                        let selection = text_pattern.GetSelection()?;
                        let caret = selection.GetElement(0)?;
                        caret.ExpandToEnclosingUnit(TextUnit_Character)?;

                        let bounds = caret.GetBoundingRectangles()?;
                        let mut elements = std::ptr::null_mut::<RECT>();
                        let mut elements_len = 0;

                        UNMANAGED.automation_instance.0.SafeArrayToRectNativeArray(
                            bounds,
                            &mut elements,
                            &mut elements_len,
                        )?;

                        if elements_len > 0 {
                            let bounds = *elements;

                            UNMANAGED.send_event(WindowEvent::Reposition {
                                x: bounds.left,
                                y: bounds.bottom,
                            });
                        }
                    },
                    ConsoleState::Accessible { caret_x, caret_y } => {
                        UNMANAGED.send_event(WindowEvent::Reposition { x: caret_x, y: caret_y });
                    },
                }
            },
        }

        Err(anyhow!("Failed to acquire caret position"))
    }
}

#[derive(Clone, Copy, Debug)]
enum ConsoleState {
    None,
    Console { hwnd: HWND },
    Accessible { caret_x: i32, caret_y: i32 },
}

#[allow(dead_code)]
struct Unmanaged {
    main_thread: u32,
    previous_focus: RwLock<Option<u32>>,
    global_state: RwLock<Option<Arc<GlobalState>>>,
    event_sender: RwLock<Option<EventLoopProxy>>,
    foreground_hook: RwLock<HWINEVENTHOOK>,
    location_hook: RwLock<Option<HWINEVENTHOOK>>,
    console_state: RwLock<ConsoleState>,
    automation_instance: AutomationTable,
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

    pub fn lookup(_name: &str) -> Option<ProcessedAsset> {
        None
    }
}

#[allow(unused_variables)]
pub async fn init(global_state: Arc<GlobalState>, proxy: EventLoopProxy) -> Result<()> {
    UNMANAGED.event_sender.write().replace(proxy);
    UNMANAGED.global_state.write().replace(global_state);

    unsafe {
        update_focused_state(GetForegroundWindow());
    }

    Ok(())
}

unsafe fn update_focused_state(hwnd: HWND) {
    if let Some(hook) = UNMANAGED.location_hook.write().take() {
        UnhookWinEvent(hook);
    }

    UNMANAGED.send_event(WindowEvent::Hide);

    let mut process_id: u32 = 0;
    let thread_id = GetWindowThreadProcessId(hwnd, &mut process_id);
    let process_handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
        Ok(process_handle) => process_handle,
        Err(e) => {
            debug!("Failed to get a handle to a Windows process, it's likely been closed: {e}");
            return;
        },
    };

    // Get the terminal name
    let mut process_name = vec![0; 256];
    let len = K32GetProcessImageFileNameA(process_handle, &mut process_name) as usize;
    process_name.truncate(len + 1);
    let title = match CStr::from_bytes_with_nul(&process_name)
        .expect("Missing null terminator")
        .to_str()
    {
        Ok(process_name) => match process_name.split('\\').last() {
            Some(title) => match title.strip_suffix(".exe") {
                Some(title) => title,
                None => return,
            },
            None => return,
        },
        Err(_) => return,
    };

    match title {
        title if ["Hyper", "Code"].contains(&title) => (),
        title if ["cmd", "mintty", "powershell", "WindowsTerminal"].contains(&title) => {
            *UNMANAGED.console_state.write() = ConsoleState::Console { hwnd }
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
            update_focused_state(hwnd)
        },
        e if e == EVENT_OBJECT_LOCATIONCHANGE
            && OBJECT_IDENTIFIER(id_object) == OBJID_WINDOW
            && id_child == CHILDID_SELF as i32 =>
        {
            UNMANAGED.send_event(WindowEvent::Hide);
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
                        .accLocation(&mut left, &mut top, &mut width, &mut height, &varchild)
                        .is_ok()
                    {
                        *UNMANAGED.console_state.write() = ConsoleState::Accessible {
                            caret_x: left,
                            caret_y: top + height,
                        }
                    }
                }
            }
        },
        _ => (),
    }
}
