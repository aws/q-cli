#![allow(non_upper_case_globals)]

mod ax_observer;
mod ui_element;
use std::ffi::c_void;
use std::hash::Hash;
use std::sync::Arc;

use accessibility_sys::{
    kAXApplicationActivatedNotification,
    kAXApplicationShownNotification,
    kAXFocusedWindowChangedNotification,
    kAXMainWindowChangedNotification,
    kAXUIElementDestroyedNotification,
    kAXWindowCreatedNotification,
    kAXWindowMovedNotification,
    kAXWindowResizedNotification,
    pid_t,
    AXError,
    AXIsProcessTrusted,
    AXObserverRef,
    AXUIElement,
    AXUIElementCreateApplication,
    AXUIElementRef,
};
use appkit_nsworkspace_bindings::{
    INSNotification,
    INSRunningApplication,
    INSWorkspace,
    NSApplicationActivationPolicy_NSApplicationActivationPolicyProhibited as ActivationPolicy_Prohibited,
    NSRunningApplication,
    NSWorkspace,
    NSWorkspaceActiveSpaceDidChangeNotification,
    NSWorkspaceDidActivateApplicationNotification,
    NSWorkspaceDidLaunchApplicationNotification,
    NSWorkspaceDidTerminateApplicationNotification,
    NSWorkspace_NSWorkspaceRunningApplications,
};
use ax_observer::{
    AXObserver,
    AccessibilityCallbackData,
};
use cocoa::base::nil;
use core_foundation::base::TCFType;
use core_foundation::string::{
    CFString,
    CFStringRef,
};
use dashmap::DashMap;
use flume::Sender;
use parking_lot::Mutex;
use tracing::{
    debug,
    error,
    info,
    trace,
    warn,
};
use ui_element::UIElement;

use super::util::notification_center::get_app_from_notification;
use super::util::{
    NSArray,
    NSString,
    NotificationCenter,
};

static BLOCKED_BUNDLE_IDS: &[&str] = &[
    "com.apple.ViewBridgeAuxiliary",
    "com.apple.notificationcenterui",
    "com.apple.WebKit.WebContent",
    "com.apple.WebKit.Networking",
    "com.apple.controlcenter",
    "com.mschrage.fig",
];

static TRACKED_NOTIFICATIONS: &[&str] = &[
    kAXWindowCreatedNotification,
    kAXFocusedWindowChangedNotification,
    kAXMainWindowChangedNotification,
    kAXApplicationShownNotification,
    kAXApplicationActivatedNotification,
    kAXWindowResizedNotification,
    kAXWindowMovedNotification,
    kAXUIElementDestroyedNotification,
];

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ApplicationSpecifier {
    pub pid: pid_t,
    pub bundle_id: String,
}

pub struct WindowServer {
    observers: DashMap<ApplicationSpecifier, AXObserver, fnv::FnvBuildHasher>,
    sender: Sender<WindowServerEvent>,
}

pub enum WindowServerEvent {
    FocusChanged {
        app: Option<ApplicationSpecifier>,
        window: Option<UIElement>,
    },
    ActiveSpaceChanged {
        is_fullscreen: bool,
    },
}

unsafe fn app_bundle_id(app: &NSRunningApplication) -> Option<String> {
    if matches!(app, NSRunningApplication(nil)) {
        return None;
    }
    let bundle_id: NSString = app.bundleIdentifier().into();
    let s: Result<&str, _> = bundle_id.try_into();
    s.ok().map(|s| s.to_owned())
}

impl WindowServer {
    pub fn new(sender: Sender<WindowServerEvent>) -> Self {
        Self {
            observers: Default::default(),
            sender,
        }
    }

    #[allow(clippy::missing_safety_doc)]
    unsafe fn register(&mut self, ns_app: NSRunningApplication, from_activation: bool) {
        if !AXIsProcessTrusted() {
            info!("Cannot register to observer window events without accessibility perms");
            return;
        }

        let bundle_id = match app_bundle_id(&ns_app) {
            Some(bundle_id) => bundle_id,
            None => {
                debug!("Ignoring empty bundle id");
                return;
            },
        };

        let pid = ns_app.processIdentifier();
        let key = ApplicationSpecifier {
            pid,
            bundle_id: bundle_id.clone(),
        };

        let ax_ref = AXUIElementCreateApplication(pid);

        for blocked_bundle in BLOCKED_BUNDLE_IDS {
            if *blocked_bundle == bundle_id {
                debug!("Ignoring bundle id {:?}", bundle_id);
                return;
            }
        }

        if ns_app.activationPolicy() == ActivationPolicy_Prohibited {
            debug!("Ignoring application by activation policy");
            return;
        }

        if self.observers.contains_key(&key) {
            debug!("app {} is already registered", key.bundle_id);
            self.deregister(&key.bundle_id)
        }

        if from_activation {
            // In Swift had 0.25s delay before this...?
            if let Ok(window) = UIElement::new_with_focused_attribute(ax_ref, &key) {
                if let Err(e) = self.sender.send(WindowServerEvent::FocusChanged {
                    app: Some(key.clone()),
                    window: Some(window),
                }) {
                    warn!("Error sending focus changed event: {e:?}");
                };
            }
        }

        let bundle_id = key.bundle_id.as_str();
        if let Ok(mut observer) = AXObserver::create(key.clone(), ax_ref, self.sender.clone(), ax_callback) {
            let result: Result<Vec<_>, AXError> = TRACKED_NOTIFICATIONS
                .iter()
                .map(|notification| observer.subscribe(notification))
                .collect();

            if result.is_ok() {
                debug!("Began tracking {bundle_id:?}");
                self.observers.insert(key, observer);
                return;
            }
        }

        warn!("Error setting up tracking for '{bundle_id:?}'");
    }

    fn deregister(&mut self, bundle_id: &str) {
        self.observers.retain(|key, _| bundle_id != key.bundle_id);
    }

    fn register_all(&mut self) {
        self.deregister_all();

        unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            let app = workspace.frontmostApplication();
            self.register(app, true);

            let apps: NSArray<NSRunningApplication> = workspace.runningApplications().into();
            for app in apps.iter() {
                self.register(NSRunningApplication(app), false)
            }
        }

        info!("Tracking {:?} applications", self.observers.len());
    }

    pub fn init(&mut self) {
        self.register_all();
    }

    fn deregister_all(&mut self) {
        self.observers.clear();
    }
}

#[allow(clippy::missing_safety_doc)]
pub unsafe fn subscribe_to_all(server: &Arc<Mutex<WindowServer>>) {
    let mut center = NotificationCenter::shared();

    // Previously (in Swift) subscribed to the following as no-ops / log only:
    // - NSWorkspaceDidDeactivateApplicationNotification

    let closure_server = server.clone();
    center.subscribe(NSWorkspaceActiveSpaceDidChangeNotification, move |notification, _| {
        let ax_ref = notification.object() as AXUIElementRef;
        let elem: UIElement = ax_ref.into();
        if let Ok(is_fullscreen) = elem.is_fullscreen() {
            let server = closure_server.lock();
            if let Err(e) = server
                .sender
                .send(WindowServerEvent::ActiveSpaceChanged { is_fullscreen })
            {
                warn!("Error sending active space changed notif: {e:?}");
            }
        }
    });

    let closure_server = server.clone();
    center.subscribe(NSWorkspaceDidActivateApplicationNotification, move |notification, _| {
        if let Some(app) = get_app_from_notification(notification) {
            let bundle_id = app_bundle_id(&app);
            trace!("Activated application {bundle_id:?}");
            let mut server = closure_server.lock();
            server.register(app, true)
        }
    });

    let closure_server = server.clone();
    center.subscribe(
        NSWorkspaceDidTerminateApplicationNotification,
        move |notification, _| {
            if let Some(ns_app) = get_app_from_notification(notification) {
                if let Some(bundle_id) = app_bundle_id(&ns_app) {
                    trace!("Terminated application - {bundle_id:?}");

                    let apps: NSArray<NSRunningApplication> =
                        NSWorkspace::sharedWorkspace().runningApplications().into();

                    let has_running = apps.iter().any(|running| {
                        let running = NSRunningApplication(running);
                        app_bundle_id(&running).map(|id| id == bundle_id).unwrap_or(false)
                    });

                    if !has_running {
                        trace!("Deregistering app {bundle_id:?} since no other instances are running");
                        let mut server = closure_server.lock();
                        server.deregister(bundle_id.as_str());
                    }
                }
            }
        },
    );

    let closure_server = server.clone();
    center.subscribe(NSWorkspaceDidLaunchApplicationNotification, move |notification, _| {
        if let Some(app) = get_app_from_notification(notification) {
            let bundle_id = app_bundle_id(&app);
            trace!("Launched application - {bundle_id:?}");
            let mut server = closure_server.lock();
            server.register(app, true)
        }
    });
}

#[no_mangle]
unsafe extern "C" fn ax_callback(
    _observer: AXObserverRef,
    element: AXUIElementRef,
    notification_name: CFStringRef,
    refcon: *mut c_void,
) {
    if refcon.is_null() {
        error!("refcon must not be null");
        return;
    }

    let cb_data: &mut AccessibilityCallbackData = &mut *(refcon as *mut AccessibilityCallbackData);
    // get_rule will call CFRetain to increment the RC in objc to make sure element is not freed
    // before we are done with it. CFRelease is called automatically on drop.
    let element = AXUIElement::wrap_under_get_rule(element);

    let name = CFString::wrap_under_get_rule(notification_name);
    let app = &cb_data.app;

    let event = match name.to_string().as_str() {
        kAXFocusedWindowChangedNotification => Some(WindowServerEvent::FocusChanged {
            app: Some(app.clone()),
            window: Some(UIElement::new_with_app(element, app)),
        }),
        kAXMainWindowChangedNotification => Some(WindowServerEvent::FocusChanged {
            app: None,
            window: Some(UIElement::new_with_app(element, app)),
        }),
        kAXApplicationActivatedNotification | kAXApplicationShownNotification => {
            UIElement::new_with_focused_attribute(cb_data.ax_ref, app)
                .ok()
                .map(|window| WindowServerEvent::FocusChanged {
                    app: Some(app.clone()),
                    window: Some(window),
                })
        },
        kAXWindowResizedNotification | kAXWindowMovedNotification => {
            // fixes issue where opening app from spotlight loses window tracking
            let frontmost = NSWorkspace::sharedWorkspace().frontmostApplication();
            let bundle_id = app_bundle_id(&frontmost);
            if bundle_id
                .as_deref()
                .map(|a| a == cb_data.app.bundle_id)
                .unwrap_or(false)
            {
                UIElement::new_with_focused_attribute(cb_data.ax_ref, app)
                    .ok()
                    .map(|window| WindowServerEvent::FocusChanged {
                        app: Some(app.clone()),
                        window: Some(window),
                    })
            } else {
                info!("Resized window ({bundle_id:?}) not associated with frontmost app.");
                None
            }
        },
        unknown => {
            info!("Unhandled AX event: {unknown}");
            None
        },
    };
    if let Some(event) = event {
        if let Err(e) = cb_data.sender.send(event) {
            warn!("Error sending focus changed event: {e:?}");
        }
    }
}
