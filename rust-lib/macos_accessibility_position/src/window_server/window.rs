use accessibility::util::ax_call;
use accessibility_sys::{
    kAXFocusedWindowAttribute,
    AXError,
    AXUIElementCopyAttributeValue,
    AXUIElementRef,
};
use appkit_nsworkspace_bindings::CFTypeRef;
use core_foundation::base::TCFType;
use core_foundation::string::CFString;

use super::ApplicationSpecifier;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Window {
    app: ApplicationSpecifier,
    ax_ref: AXUIElementRef,
}

// SAFETY: Pointer AXUIElementRef is send + sync safe
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

impl Window {
    pub fn new(ax_ref: AXUIElementRef, app: &ApplicationSpecifier) -> Self {
        Self {
            app: app.clone(),
            ax_ref,
        }
    }

    pub unsafe fn new_with_focused_attribute(
        source_ref: AXUIElementRef,
        app: &ApplicationSpecifier,
    ) -> Result<Self, AXError> {
        let window = ax_call(|window: *mut CFTypeRef| {
            let attr = CFString::from_static_string(kAXFocusedWindowAttribute);
            let attr_ref = attr.as_concrete_TypeRef();
            AXUIElementCopyAttributeValue(source_ref, attr_ref, window)
        });

        window.map(|window| Self::new(window as AXUIElementRef, app))
    }
}
