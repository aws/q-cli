use std::boxed::Box;
use std::ffi::c_void;
use std::pin::Pin;

use accessibility::util::ax_call;
use accessibility_sys::{
    AXError,
    AXObserverAddNotification,
    AXObserverCallback,
    AXObserverCreate,
    AXObserverGetRunLoopSource,
    AXObserverRef,
    AXUIElementRef,
};
use core_foundation::base::TCFType;
use core_foundation::runloop::{
    kCFRunLoopDefaultMode,
    CFRunLoopAddSource,
    CFRunLoopGetCurrent,
    CFRunLoopRemoveSource,
};
use core_foundation::string::{
    CFString,
    CFStringRef,
};
use flume::Sender;

use super::{
    ApplicationSpecifier,
    WindowServerEvent,
};

pub struct AccessibilityCallbackData {
    pub app: ApplicationSpecifier,
    pub ax_ref: AXUIElementRef,
    pub sender: Sender<WindowServerEvent>,
}

pub struct AXObserver {
    inner: AXObserverRef,
    callback_data: Pin<Box<AccessibilityCallbackData>>,
}

// SAFETY: Pointers AXObserverRef, AXUIElementRef is send + sync safe
unsafe impl Send for AXObserver {}
unsafe impl Sync for AXObserver {}

impl AXObserver {
    pub unsafe fn create(
        app: ApplicationSpecifier,
        ax_ref: AXUIElementRef,
        sender: Sender<WindowServerEvent>,
        callback: AXObserverCallback,
    ) -> Result<Self, AXError> {
        let observer = ax_call(|x: *mut AXObserverRef| AXObserverCreate(app.pid, callback, x))?;

        CFRunLoopAddSource(
            CFRunLoopGetCurrent(),
            AXObserverGetRunLoopSource(observer),
            kCFRunLoopDefaultMode,
        );

        Ok(Self {
            inner: observer,
            callback_data: Box::pin(AccessibilityCallbackData { app, ax_ref, sender }),
        })
    }

    pub unsafe fn subscribe(&mut self, ax_event: &str) -> Result<(), AXError> {
        ax_call(|_x: *mut c_void| {
            let callback_data: *const AccessibilityCallbackData = &*self.callback_data;
            AXObserverAddNotification(
                self.inner,
                self.callback_data.ax_ref,
                CFString::from(ax_event).as_CFTypeRef() as CFStringRef,
                callback_data as *const _ as *mut c_void,
            )
        })
        .map(|_| ())
    }
}

impl Drop for AXObserver {
    fn drop(&mut self) {
        unsafe {
            CFRunLoopRemoveSource(
                CFRunLoopGetCurrent(),
                AXObserverGetRunLoopSource(self.inner),
                kCFRunLoopDefaultMode,
            );
        }
    }
}
