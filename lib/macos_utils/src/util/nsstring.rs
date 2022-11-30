use appkit_nsworkspace_bindings::NSString as AppkitNSString;
use cocoa::base::nil as NIL;
use cocoa::foundation::NSString as CocoaNSString;
use objc::runtime::Object;

use super::{
    Id,
    IdRef,
};

/// This is an owned NSString
#[repr(transparent)]
pub struct NSString(Id);

impl NSString {
    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn new(raw: *mut Object) -> Self {
        Self(Id::new(raw))
    }

    pub fn into_inner(self) -> Id {
        self.0
    }

    pub fn to_appkit_nsstring(self) -> AppkitNSString {
        AppkitNSString(self.0.autorelease())
    }

    pub fn as_str(&self) -> Option<&str> {
        if self.is_null() {
            None
        } else {
            unsafe {
                let bytes: *const std::os::raw::c_char = self.UTF8String();
                let len = self.len();
                let bytes = std::slice::from_raw_parts(bytes as *const u8, len);
                Some(std::str::from_utf8_unchecked(bytes))
            }
        }
    }
}

impl From<AppkitNSString> for NSString {
    fn from(s: AppkitNSString) -> Self {
        Self(unsafe { Id::new(s.0) })
    }
}

impl From<&str> for NSString {
    fn from(s: &str) -> Self {
        let inner = unsafe { CocoaNSString::alloc(NIL).init_str(s) };
        Self(unsafe { Id::new(inner) })
    }
}

impl std::ops::Deref for NSString {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// This is a borrowed NSString
#[repr(transparent)]
pub struct NSStringRef(IdRef);

impl NSStringRef {
    /// # Safety
    ///
    /// This is unsafe because the caller must ensure that the pointer is valid
    /// for the lifetime of the returned object.
    pub unsafe fn new(inner: *const Object) -> Self {
        Self(IdRef::new(inner))
    }

    pub fn as_str(&self) -> Option<&str> {
        if self.0.is_nil() {
            None
        } else {
            unsafe {
                let obj = *self.0 as *mut Object;
                let bytes: *const std::os::raw::c_char = obj.UTF8String();
                let len = obj.len();
                let bytes = std::slice::from_raw_parts(bytes as *const u8, len);
                Some(std::str::from_utf8_unchecked(bytes))
            }
        }
    }
}

impl std::ops::Deref for NSStringRef {
    type Target = *const Object;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<AppkitNSString> for NSStringRef {
    fn from(s: AppkitNSString) -> Self {
        Self(unsafe { IdRef::new(s.0) })
    }
}
