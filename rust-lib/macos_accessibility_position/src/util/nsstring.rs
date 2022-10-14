use std::convert::TryFrom;

use appkit_nsworkspace_bindings::{
    id,
    NSString as AppkitNSString,
    NSString_NSStringExtensionMethods,
    NSUTF8StringEncoding,
};
use cocoa::base::nil as NIL;
use cocoa::foundation::{
    NSAutoreleasePool,
    NSString as CocoaNSString,
};
use objc::runtime::Object;

pub struct NSString(*mut Object);

impl NSString {
    pub(crate) fn id(&self) -> id {
        self.0
    }
}

impl From<AppkitNSString> for NSString {
    fn from(s: AppkitNSString) -> Self {
        Self(s.0)
    }
}

impl From<*mut Object> for NSString {
    fn from(s: *mut Object) -> Self {
        Self(s)
    }
}

impl From<&str> for NSString {
    fn from(s: &str) -> Self {
        let inner = unsafe { CocoaNSString::alloc(NIL).init_str(s).autorelease() };
        inner.into()
    }
}

impl From<NSString> for AppkitNSString {
    fn from(s: NSString) -> Self {
        AppkitNSString(s.id())
    }
}

impl From<NSString> for *mut Object {
    fn from(s: NSString) -> Self {
        s.id()
    }
}

impl TryFrom<NSString> for &str {
    type Error = &'static str;

    fn try_from(s: NSString) -> Result<Self, Self::Error> {
        let s: AppkitNSString = s.into();
        if matches!(s, AppkitNSString(NIL)) {
            Err("Cannot convert nil NSString")
        } else {
            unsafe {
                let bytes: *const std::os::raw::c_char = s.UTF8String();
                let len = s.lengthOfBytesUsingEncoding_(NSUTF8StringEncoding);
                let bytes = std::slice::from_raw_parts(bytes as *const u8, len as usize);
                Ok(std::str::from_utf8_unchecked(bytes))
            }
        }
    }
}
