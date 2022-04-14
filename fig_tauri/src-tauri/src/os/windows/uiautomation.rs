use crate::api::window::handle_ui_state;
use crate::state::{Rect, UIState};
use std::convert::TryInto;
use windows::Win32::{
    Foundation::{BOOL, BSTR, HWND, RECT},
    Graphics::Gdi::{self, GetMonitorInfoW, MonitorFromRect, HMONITOR, MONITORINFO},
    System::{
        Com::{self, CLSCTX_INPROC_SERVER, VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0},
        Ole::VT_BSTR,
    },
    UI::{
        Accessibility::{self, IUIAutomation, IUIAutomationElement, TreeScope},
        WindowsAndMessaging,
    },
};

impl TryInto<Rect> for RECT {
    type Error = &'static str;

    fn try_into(self) -> Result<Rect, Self::Error> {
        Ok(Rect {
            x: self.left,
            y: self.bottom,
            width: self.right - self.left,
            height: self.bottom - self.top,
        })
    }
}

pub fn ui_listener_event_loop() {
    unsafe { Com::CoInitialize(std::ptr::null()).unwrap() };
    let automation: IUIAutomation =
        unsafe { Com::CoCreateInstance(&Accessibility::CUIAutomation, None, CLSCTX_INPROC_SERVER) }
            .unwrap();

    loop {
        macro_rules! skip_fail {
            ($res:expr) => {
                match $res {
                    Ok(val) => val,
                    Err(_) => {
                        handle_ui_state(UIState::Unfocused);
                        continue;
                    }
                }
            };
        }

        macro_rules! skip_none {
            ($opt:expr) => {
                match $opt {
                    Some(val) => val,
                    None => {
                        handle_ui_state(UIState::Unfocused);
                        continue;
                    }
                }
            };
        }

        let hwnd: HWND = unsafe { WindowsAndMessaging::GetForegroundWindow() };
        let hwnd_elt: IUIAutomationElement =
            skip_fail!(unsafe { automation.clone().ElementFromHandle(hwnd) });

        let document_rect = skip_none!(unsafe {
            get_automation_elt_rect(
                hwnd_elt.clone(),
                automation.clone(),
                "Hyper",
                Accessibility::UIA_NamePropertyId,
                Accessibility::TreeScope_Children,
            )
        });

        let caret_rect = skip_none!(unsafe {
            get_automation_elt_rect(
                hwnd_elt.clone(),
                automation.clone(),
                "edit",
                Accessibility::UIA_LocalizedControlTypePropertyId,
                Accessibility::TreeScope_Descendants,
            )
        });

        let banner_rect = skip_none!(unsafe {
            get_automation_elt_rect(
                hwnd_elt.clone(),
                automation.clone(),
                "banner",
                Accessibility::UIA_LocalizedControlTypePropertyId,
                Accessibility::TreeScope_Descendants,
            )
        });

        let win_rect = skip_none!(unsafe { get_hwnd_rect(hwnd) });
        let screen_rect = skip_none!(unsafe { get_screen_rect(&caret_rect) });

        let document_rect: Rect = document_rect.try_into().unwrap();
        let mut caret_rect: Rect = caret_rect.try_into().unwrap();
        let banner_rect: Rect = banner_rect.try_into().unwrap();
        let win_rect: Rect = win_rect.try_into().unwrap();
        let screen_rect: Rect = screen_rect.try_into().unwrap();

        caret_rect.x -= banner_rect.x - document_rect.x;

        handle_ui_state(UIState::Focused {
            caret: caret_rect,
            window: win_rect,
            screen: screen_rect,
        });
    }
}

unsafe fn get_screen_rect(caret_rect: &RECT) -> Option<RECT> {
    let screen: HMONITOR =
        MonitorFromRect(caret_rect as *const RECT, Gdi::MONITOR_DEFAULTTONEAREST);

    let mut monitor_info: MONITORINFO = Default::default();
    monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;

    if GetMonitorInfoW(screen, &mut monitor_info as *mut MONITORINFO) == BOOL(0) {
        return None;
    }

    Some(monitor_info.rcWork)
}

unsafe fn get_automation_elt_rect(
    hwnd_elt: IUIAutomationElement,
    automation: IUIAutomation,
    control_type_property_id: &str,
    property_id: i32,
    tree_scope: TreeScope,
) -> Option<RECT> {
    let pattern_condition = automation
        .CreatePropertyCondition(
            property_id,
            get_matching_condition(control_type_property_id),
        )
        .ok()?;
    let elt: IUIAutomationElement = hwnd_elt.FindFirst(tree_scope, pattern_condition).ok()?;

    elt.CurrentBoundingRectangle().ok()
}

unsafe fn get_hwnd_rect(hwnd: HWND) -> Option<RECT> {
    let mut win_rect: RECT = Default::default();
    if WindowsAndMessaging::GetWindowRect(hwnd, &mut win_rect as *mut RECT) == BOOL(0) {
        return None;
    }
    Some(win_rect)
}

fn get_matching_condition(localized_control_type: &str) -> VARIANT {
    let byte_arr: &[u8] = localized_control_type.as_bytes();

    let mut short_vec: Vec<u16> = Vec::new();
    for &i in byte_arr {
        short_vec.push(i as u16);
    }

    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: std::mem::ManuallyDrop::new(VARIANT_0_0 {
                vt: VT_BSTR.0 as u16,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: VARIANT_0_0_0 {
                    bstrVal: std::mem::ManuallyDrop::new(BSTR::from_wide(&short_vec[..])),
                },
            }),
        },
    }
}
