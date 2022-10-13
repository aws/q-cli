use std::sync::Arc;

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
use objc::runtime::Object;
use parking_lot::Mutex;

use super::NSString;

pub struct Subscription {
    observer: Mutex<Option<*mut Object>>,
    center: NSNotificationCenter,
}

// SAFETY: Pointer for *mut Object is send + sync
unsafe impl Send for Subscription {}
unsafe impl Sync for Subscription {}

impl Subscription {
    pub fn empty(center: NSNotificationCenter) -> Self {
        Self {
            observer: Mutex::new(None::<*mut Object>),
            center,
        }
    }

    pub fn set_observer(&mut self, observer: *mut Object) {
        self.observer.lock().replace(observer);
    }

    pub fn cancel(&mut self) {
        if let Some(observer) = self.observer.lock().take() {
            unsafe {
                self.center.removeObserver_(observer);
            }
        }
    }
}

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

    pub fn distributed() -> Self {
        use objc::{
            class,
            msg_send,
            sel,
            sel_impl,
        };
        let distributed_default: *mut Object =
            unsafe { msg_send![class!(NSDistributedNotificationCenter), defaultCenter] };
        Self::new(appkit_nsworkspace_bindings::NSNotificationCenter(distributed_default))
    }

    pub fn subscribe<F>(&mut self, notification_name: impl Into<AppkitNSString>, mut f: F)
    where
        F: FnMut(NSNotification, Arc<Mutex<Subscription>>),
    {
        let subscription = Arc::new(Mutex::new(Subscription::empty(self.inner)));
        let block_sub = subscription.clone();
        let mut block = block::ConcreteBlock::new(move |notif: NSNotification| {
            f(notif, block_sub.clone());
        });
        unsafe {
            // addObserverForName copies block for us.
            let observer = self.inner.addObserverForName_object_queue_usingBlock_(
                notification_name.into(),
                NIL,
                NSOperationQueue(NIL),
                &mut block as *mut _ as *mut std::os::raw::c_void,
            ) as *mut Object;
            let mut subscription = subscription.lock();
            subscription.set_observer(observer);
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
