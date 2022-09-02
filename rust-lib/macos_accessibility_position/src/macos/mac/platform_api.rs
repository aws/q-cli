use std::ffi::{
    c_void,
    CStr,
};
use std::str;

use appkit_nsworkspace_bindings::{
    INSRunningApplication,
    INSWorkspace,
    NSString_NSStringExtensionMethods,
    NSWorkspace,
};
use core_foundation::base::{
    CFGetTypeID,
    ToVoid,
};
use core_foundation::boolean::CFBooleanGetTypeID;
use core_foundation::dictionary::CFDictionaryGetTypeID;
use core_foundation::mach_port::CFTypeID;
use core_foundation::number::{
    CFBooleanGetValue,
    CFNumberGetType,
    CFNumberGetTypeID,
    CFNumberGetValue,
    CFNumberRef,
    CFNumberType,
};
use core_foundation::string::{
    kCFStringEncodingUTF8,
    CFString,
    CFStringGetCStringPtr,
    CFStringGetTypeID,
};
use core_graphics::display::*;

use super::core_graphics_patch::CGRectMakeWithDictionaryRepresentation;
use super::window_position::FromCgRect;
use crate::macos::general::active_window::ActiveWindow;
use crate::macos::general::platform_api::PlatformApi;
use crate::macos::general::window_position::WindowPosition;

#[allow(non_upper_case_globals)]
pub const kCFNumberSInt32Type: CFNumberType = 3;
#[allow(non_upper_case_globals)]
pub const kCFNumberSInt64Type: CFNumberType = 4;

#[derive(Debug)]
enum DictEntryValue {
    Number(i64),
    Bool(bool),
    String(String),
    Rect(WindowPosition),
    Unknown,
}

pub struct MacPlatformApi {}

impl PlatformApi for MacPlatformApi {
    fn get_position(&self) -> Result<WindowPosition, ()> {
        if let Ok(active_window) = self.get_active_window() {
            return Ok(active_window.position);
        }

        Err(())
    }

    fn get_active_window(&self) -> Result<ActiveWindow, ()> {
        const OPTIONS: CGWindowListOption = kCGWindowListOptionOnScreenOnly | kCGWindowListExcludeDesktopElements;
        let window_list_info = unsafe { CGWindowListCopyWindowInfo(OPTIONS, kCGNullWindowID) };

        let count: isize = unsafe { CFArrayGetCount(window_list_info) };

        let active_app;
        unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            active_app = workspace.frontmostApplication();
        };

        let active_window_pid = unsafe { active_app.processIdentifier() as i64 };

        let bundle_id: String = unsafe {
            let bundle_id_cfstring = active_app.bundleIdentifier();
            let c_buf: *const ::std::os::raw::c_char = bundle_id_cfstring.UTF8String();
            let c_str: &CStr = CStr::from_ptr(c_buf);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice.to_owned()
        };

        let mut win_pos = WindowPosition::new(0., 0., 0., 0.);

        for i in 0..count - 1 {
            let dic_ref = unsafe { CFArrayGetValueAtIndex(window_list_info, i) as CFDictionaryRef };
            let window_pid = get_from_dict(dic_ref, "kCGWindowOwnerPID");

            if let DictEntryValue::Number(win_pid) = window_pid {
                if win_pid != active_window_pid {
                    continue;
                }

                if let DictEntryValue::Rect(window_bounds) = get_from_dict(dic_ref, "kCGWindowBounds") {
                    if window_bounds.width < 50. || window_bounds.height < 50. {
                        continue;
                    }

                    win_pos = window_bounds;
                }

                if let DictEntryValue::Number(window_id) = get_from_dict(dic_ref, "kCGWindowNumber") {
                    let active_window = ActiveWindow {
                        window_id: window_id.to_string(),
                        process_id: active_window_pid as u64,
                        position: win_pos,
                        bundle_id,
                    };
                    return Ok(active_window);
                }
            }
        }

        unsafe { CFRelease(window_list_info as CFTypeRef) }

        Err(())
    }
}

fn get_from_dict(dict: CFDictionaryRef, key: &str) -> DictEntryValue {
    let cf_key: CFString = key.into();
    let mut value: *const c_void = std::ptr::null();
    if unsafe { CFDictionaryGetValueIfPresent(dict, cf_key.to_void(), &mut value) != 0 } {
        let type_id: CFTypeID = unsafe { CFGetTypeID(value) };
        if type_id == unsafe { CFNumberGetTypeID() } {
            let value = value as CFNumberRef;

            #[allow(non_upper_case_globals)]
            match unsafe { CFNumberGetType(value) } {
                kCFNumberSInt64Type => {
                    let mut value_i64 = 0_i64;
                    let out_value: *mut i64 = &mut value_i64;
                    let converted = unsafe { CFNumberGetValue(value, kCFNumberSInt64Type, out_value.cast()) };
                    if converted {
                        return DictEntryValue::Number(value_i64);
                    }
                },
                kCFNumberSInt32Type => {
                    let mut value_i32 = 0_i32;
                    let out_value: *mut i32 = &mut value_i32;
                    let converted = unsafe { CFNumberGetValue(value, kCFNumberSInt32Type, out_value.cast()) };
                    if converted {
                        return DictEntryValue::Number(value_i32 as i64);
                    }
                },
                n => {
                    eprintln!("Unsupported Number of typeId: {}", n);
                },
            }
        } else if type_id == unsafe { CFBooleanGetTypeID() } {
            return DictEntryValue::Bool(unsafe { CFBooleanGetValue(value.cast()) });
        } else if type_id == unsafe { CFStringGetTypeID() } {
            let c_ptr = unsafe { CFStringGetCStringPtr(value.cast(), kCFStringEncodingUTF8) };
            return if !c_ptr.is_null() {
                let c_result = unsafe { CStr::from_ptr(c_ptr) };
                let result = String::from(c_result.to_str().unwrap());
                DictEntryValue::String(result)
            } else {
                DictEntryValue::Unknown
            };
        } else if type_id == unsafe { CFDictionaryGetTypeID() } && key == "kCGWindowBounds" {
            let rect: CGRect = unsafe {
                let mut rect = std::mem::zeroed();
                CGRectMakeWithDictionaryRepresentation(value.cast(), &mut rect);
                rect
            };

            return DictEntryValue::Rect(WindowPosition::from_cg_rect(&rect));
        } else {
            eprintln!("Unexpected type: {}", type_id);
        }
    }

    DictEntryValue::Unknown
}
