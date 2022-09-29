#![allow(non_upper_case_globals)]
use std::ffi::c_void;

use core_foundation_sys::base::CFTypeID;

pub type AXValueType = u32;
pub const kAXValueTypeCGPoint: u32 = 1;
pub const kAXValueTypeCGSize: u32 = 2;
pub const kAXValueTypeCGRect: u32 = 3;
pub const kAXValueTypeCFRange: u32 = 4;
pub const kAXValueTypeAXError: u32 = 5;
pub const kAXValueTypeIllegal: u32 = 0;

pub enum __AXValue {}
pub type AXValueRef = *mut __AXValue;

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    pub fn AXValueGetTypeID() -> CFTypeID;
    pub fn AXValueCreate(theType: AXValueType, valuePtr: *const c_void) -> AXValueRef;
    pub fn AXValueGetType(value: AXValueRef) -> AXValueType;
    pub fn AXValueGetValue(value: AXValueRef, theType: AXValueType, valuePtr: *mut c_void) -> bool;
}
