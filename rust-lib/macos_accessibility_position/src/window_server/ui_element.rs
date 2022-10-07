use accessibility::util::ax_call;
use accessibility_sys::{
    kAXFocusedWindowAttribute,
    kAXFullScreenAttribute,
    AXError,
    AXUIElement,
    AXUIElementCopyAttributeValue,
    AXUIElementRef,
};
use appkit_nsworkspace_bindings::CFTypeRef;
use core_foundation::base::TCFType;
use core_foundation::boolean::{
    CFBoolean,
    CFBooleanRef,
};
use core_foundation::string::CFString;

use super::ApplicationSpecifier;

#[allow(dead_code)]
pub struct UIElement {
    app: Option<ApplicationSpecifier>,
    ax_ref: AXUIElement,
}

// SAFETY: Pointer AXUIElement is send + sync safe
unsafe impl Send for UIElement {}
unsafe impl Sync for UIElement {}

impl From<AXUIElement> for UIElement {
    fn from(ax_ref: AXUIElement) -> Self {
        Self { app: None, ax_ref }
    }
}

impl From<AXUIElementRef> for UIElement {
    fn from(ax_ref: AXUIElementRef) -> Self {
        let ax_ref = unsafe { AXUIElement::wrap_under_get_rule(ax_ref) };
        Self { app: None, ax_ref }
    }
}

impl UIElement {
    pub fn new_with_app(ax_ref: AXUIElement, app: &ApplicationSpecifier) -> Self {
        Self {
            app: Some(app.clone()),
            ax_ref,
        }
    }

    fn get_ref(&self) -> AXUIElementRef {
        self.ax_ref.as_concrete_TypeRef()
    }

    pub unsafe fn is_fullscreen(&self) -> Result<bool, AXError> {
        if self.get_ref().is_null() {
            return Err(-1);
        }

        let is_fullscreen = ax_call(|fullscreen: *mut CFTypeRef| {
            let attr = CFString::from_static_string(kAXFullScreenAttribute);
            let attr_ref = attr.as_concrete_TypeRef();
            AXUIElementCopyAttributeValue(self.get_ref(), attr_ref, fullscreen)
        })?;

        let res: bool = CFBoolean::wrap_under_get_rule(is_fullscreen as CFBooleanRef).into();
        Ok(res)
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

        window.map(|window| {
            let elem = AXUIElement::wrap_under_get_rule(window as AXUIElementRef);
            Self::new_with_app(elem, app)
        })
    }
}
