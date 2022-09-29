use std::ffi::c_void;
use std::mem::MaybeUninit;

use accessibility::util::ax_call;
use accessibility_sys::{
    kAXApplicationActivatedNotification,
    kAXApplicationDeactivatedNotification,
    kAXApplicationHiddenNotification,
    kAXApplicationShownNotification,
    kAXFocusedUIElementChangedNotification,
    kAXFocusedWindowChangedNotification,
    kAXMainWindowChangedNotification,
    kAXTitleChangedNotification,
    kAXUIElementDestroyedNotification,
    kAXWindowCreatedNotification,
    kAXWindowDeminiaturizedNotification,
    kAXWindowMiniaturizedNotification,
    kAXWindowMovedNotification,
    kAXWindowResizedNotification,
    AXObserverAddNotification,
    AXObserverCallback,
    AXObserverCreate,
    AXObserverRef,
    AXUIElementCreateApplication,
};
use appkit_nsworkspace_bindings::{
    INSRunningApplication,
    INSWorkspace,
    NSWorkspace,
};
use core_foundation::base::TCFType;
use core_foundation::string::{
    CFString,
    CFStringRef,
};

use crate::general::window_server::WindowServer;

pub struct WindowServerApi {}

impl WindowServer for WindowServerApi {
    unsafe fn register_observer(&self) -> bool {
        // TODO: Replace Callback
        let callback: MaybeUninit<AXObserverCallback> = MaybeUninit::uninit();
        let workspace = NSWorkspace::sharedWorkspace();
        let active_app = workspace.frontmostApplication();
        let pid = active_app.processIdentifier();
        let tracked_notifications = [
            kAXWindowCreatedNotification,
            kAXFocusedWindowChangedNotification,
            kAXMainWindowChangedNotification,
            kAXWindowMiniaturizedNotification,
            kAXWindowDeminiaturizedNotification,
            kAXApplicationShownNotification,
            kAXApplicationHiddenNotification,
            kAXApplicationActivatedNotification,
            kAXApplicationDeactivatedNotification,
            kAXWindowResizedNotification,
            kAXWindowMovedNotification,
            kAXUIElementDestroyedNotification,
            kAXFocusedUIElementChangedNotification,
            kAXTitleChangedNotification,
        ];

        let observer_result = ax_call(|x: *mut AXObserverRef| AXObserverCreate(pid, *callback.as_ptr(), x));
        let observer = if let Ok(observer) = observer_result {
            println!("Success");
            observer
        } else {
            println!("error {:?}", observer_result.err());
            return false;
        };
        // TODO: Fix Refcon to match macos app
        let self_ptr: MaybeUninit<*mut c_void> = MaybeUninit::uninit();
        let ax_app_ref = AXUIElementCreateApplication(pid);
        for notification in tracked_notifications {
            let add_notif = ax_call(|_x: *mut c_void| {
                AXObserverAddNotification(
                    observer,
                    ax_app_ref,
                    CFString::from(notification).as_CFTypeRef() as CFStringRef,
                    self_ptr.as_ptr() as *const *mut _ as *mut c_void,
                )
            });
            if add_notif.is_err() {
                println!("error {:?}", add_notif.err());
            }
        }

        true
    }

    // unsafe fn ax_callback(observer: AXObserverRef, element: AXUIElement, notification_name: CFString,
    // refcon: *mut c_void) : AXObserverCallback {

    // }

    unsafe fn deregister_observer() {}
}
