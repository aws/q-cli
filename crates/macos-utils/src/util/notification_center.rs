use appkit_nsworkspace_bindings::{
    INSNotification,
    INSNotificationCenter,
    INSWorkspace,
    NSNotification,
    NSNotificationCenter,
    NSRunningApplication,
    NSWorkspace,
};
use block;
use cocoa::base::{
    id,
    nil as NIL,
};
use cocoa::foundation::NSDictionary;
use objc::runtime::Object;

use super::NSString;

pub struct NotificationCenter {
    inner: NSNotificationCenter,
}

impl NotificationCenter {
    pub fn new(center: NSNotificationCenter) -> Self {
        Self { inner: center }
    }

    pub fn workspace_center() -> Self {
        let shared = unsafe { NSWorkspace::sharedWorkspace().notificationCenter() };
        Self::new(shared)
    }

    pub fn default_center() -> Self {
        let default = unsafe { msg_send![class!(NSNotificationCenter), defaultCenter] };
        Self::new(default)
    }

    pub fn distributed_center() -> Self {
        let distributed_default: *mut Object =
            unsafe { msg_send![class!(NSDistributedNotificationCenter), defaultCenter] };
        Self::new(appkit_nsworkspace_bindings::NSNotificationCenter(distributed_default))
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn remove_observer(&self, observer: id) {
        self.inner.removeObserver_(observer);
    }

    pub fn post_notification<I, K, V>(&self, notification_name: impl Into<NSString>, info: I)
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<NSString>,
        V: Into<NSString>,
    {
        let name: NSString = notification_name.into();
        unsafe {
            let (keys, objs): (Vec<id>, Vec<id>) = info
                .into_iter()
                .map(|(k, v)| (k.into().into_inner().autorelease(), v.into().into_inner().autorelease()))
                .unzip();

            use cocoa::foundation as cf;
            let keys_array = cf::NSArray::arrayWithObjects(NIL, &keys);
            let objs_array = cf::NSArray::arrayWithObjects(NIL, &objs);

            let user_info = NSDictionary::dictionaryWithObjects_forKeys_(NIL, objs_array, keys_array);

            self.inner.postNotificationName_object_userInfo_(
                name.to_appkit_nsstring(),
                NIL,
                appkit_nsworkspace_bindings::NSDictionary(user_info),
            );
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn subscribe_with_observer(
        &mut self,
        notification_name: impl Into<NSString>,
        observer: id,
        callback: objc::runtime::Sel,
    ) {
        let name: NSString = notification_name.into();
        self.inner
            .addObserver_selector_name_object_(observer, callback, name.to_appkit_nsstring(), NIL);
    }

    pub fn subscribe<F>(&mut self, notification_name: impl Into<NSString>, queue: Option<id>, f: F)
    where
        F: Fn(NSNotification),
    {
        let mut block = block::ConcreteBlock::new(f);
        unsafe {
            let name: NSString = notification_name.into();
            // addObserverForName copies block for us.
            self.inner.addObserverForName_object_queue_usingBlock_(
                name.to_appkit_nsstring(),
                NIL,
                appkit_nsworkspace_bindings::NSOperationQueue(queue.unwrap_or(NIL)),
                &mut block as *mut _ as *mut std::os::raw::c_void,
            );
        }
    }
}

pub unsafe fn get_app_from_notification(notification: &NSNotification) -> Option<NSRunningApplication> {
    let user_info = notification.userInfo().0;

    if user_info.is_null() {
        return None;
    }

    let bundle_id_str: NSString = "NSWorkspaceApplicationKey".into();

    let app = user_info.objectForKey_(***bundle_id_str);
    if app.is_null() {
        None
    } else {
        Some(NSRunningApplication(app))
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn get_user_info_from_notification(notification: &NSNotification) -> Option<id> {
    let user_info = notification.userInfo().0;

    if user_info.is_null() {
        return None;
    }

    Some(user_info)
}
