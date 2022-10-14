use appkit_nsworkspace_bindings::{
    id as Id,
    NSURL as AppkitNSURL,
};
use cocoa::foundation::{
    NSAutoreleasePool,
    NSURL as CocoaNSURL,
};
use objc::runtime::Object;

use crate::NSString;

pub struct NSURL(*mut Object);

impl NSURL {
    fn id(&self) -> Id {
        self.0
    }
}

impl<S> From<S> for NSURL
where
    S: Into<NSString>,
{
    fn from(s: S) -> Self {
        let string = s.into();
        let nsurl = AppkitNSURL::alloc();
        let obj = unsafe { nsurl.0.initWithString_(string.id()).autorelease() };
        assert!(!obj.is_null());
        Self(obj)
    }
}

impl From<NSURL> for AppkitNSURL {
    fn from(s: NSURL) -> Self {
        AppkitNSURL(s.id())
    }
}

impl std::ops::Deref for NSURL {
    type Target = Object;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.id() }
    }
}
