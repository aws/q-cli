use cocoa::base::nil;
use cocoa::foundation::NSURL as CocoaNSURL;

use super::Id;
use crate::NSString;

/// An owned NSURL
#[repr(transparent)]
#[derive(Clone)]
pub struct NSURL(Id);

impl<S> From<S> for NSURL
where
    S: Into<NSString>,
{
    fn from(s: S) -> Self {
        let string: NSString = s.into();
        let nsurl = unsafe { CocoaNSURL::alloc(nil).initWithString_(***string) };
        assert!(!nsurl.is_null());
        Self(unsafe { Id::new(nsurl) })
    }
}

impl std::ops::Deref for NSURL {
    type Target = Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
