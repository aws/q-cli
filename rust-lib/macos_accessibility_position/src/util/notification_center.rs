use appkit_nsworkspace_bindings::{
    INSDictionary,
    INSNotification,
    INSNotificationCenter,
    INSWorkspace,
    NSDictionary,
    NSNotification,
    NSNotificationCenter,
    NSOperationQueue,
    NSRunningApplication,
    NSString as AppkitNSString,
    NSWorkspace,
};
use block;
use cocoa::base::nil as NIL;

use super::NSString;

pub struct NotificationCenter {
    inner: NSNotificationCenter,
}

impl NotificationCenter {
    pub fn new(center: NSNotificationCenter) -> Self {
        Self { inner: center }
    }

    pub fn shared() -> Self {
        let shared = unsafe { NSWorkspace::sharedWorkspace().notificationCenter() };
        Self::new(shared)
    }

    pub fn subscribe<F>(&mut self, notification_name: impl Into<AppkitNSString>, f: F)
    where
        F: FnMut(NSNotification),
    {
        let mut block = block::ConcreteBlock::new(f);
        unsafe {
            // addObserverForName copies block for us.
            self.inner.addObserverForName_object_queue_usingBlock_(
                notification_name.into(),
                NIL,
                NSOperationQueue(NIL),
                &mut block as *mut _ as *mut std::os::raw::c_void,
            );
        }
    }
}

pub unsafe fn get_app_from_notification(notification: NSNotification) -> Option<NSRunningApplication> {
    let user_info = notification.userInfo();
    if let NSDictionary(NIL) = user_info {
        return None;
    }

    let bundle_id_str: NSString = "NSWorkspaceApplicationKey".into();

    let app = <NSDictionary as INSDictionary<NSString, appkit_nsworkspace_bindings::id>>::objectForKey_(
        &user_info,
        bundle_id_str.into(),
    );
    if app == NIL {
        None
    } else {
        Some(NSRunningApplication(app))
    }
}
