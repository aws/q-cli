use cocoa::base::id;
use cocoa::foundation::NSInteger;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NSOperatingSystemVersion {
    pub major: NSInteger,
    pub minor: NSInteger,
    pub patch: NSInteger,
}

impl NSOperatingSystemVersion {
    pub fn get() -> Self {
        unsafe {
            let process_info: id = msg_send![class!(NSProcessInfo), processInfo];
            msg_send![process_info, operatingSystemVersion]
        }
    }
}

impl std::fmt::Display for NSOperatingSystemVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operating_system_version() {
        let v = NSOperatingSystemVersion::get();
        assert!(v.major >= 10);

        let formatted = format!("{v}");
        assert!(formatted.contains('.'));
    }
}
