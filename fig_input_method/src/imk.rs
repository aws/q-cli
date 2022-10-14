use std::ffi::CStr;

use cocoa::base::{
    id,
    BOOL,
    NO,
    YES,
};
use cocoa::foundation::{
    NSPoint,
    NSRect,
    NSSize,
};
use fig_ipc::local::send_hook_to_socket;
use fig_proto::hooks::new_cursor_position_hook;
use macos_accessibility_position::{
    NSStringRef,
    NotificationCenter,
};
use objc::declare::ClassDecl;
use objc::runtime::{
    sel_getName,
    Class,
    Object,
    Sel,
};
use tracing::{
    debug,
    info,
    trace,
    warn,
};

#[link(name = "InputMethodKit", kind = "framework")]
extern "C" {}

// TODO: create trait IMKServer
pub unsafe fn connect_imkserver(name: id /* NSString */, identifier: id /* NSString */) {
    info!("connecting to imkserver");
    let server_alloc: id = msg_send![class!(IMKServer), alloc];
    let _server: id = msg_send![server_alloc, initWithName:name bundleIdentifier:identifier];
    info!("connected to imkserver");
}

pub fn register_controller() {
    let input_controller_class: &str = match option_env!("InputMethodServerControllerClass") {
        Some(input_controller_class) => input_controller_class,
        None => unreachable!("Must specify `InputMethodServerControllerClass` environment variable"),
    };
    info!("registering {input_controller_class}...");

    let super_class = class!(IMKInputController);
    let mut decl = ClassDecl::new(input_controller_class, super_class).unwrap();

    unsafe {
        decl.add_ivar::<BOOL>("is_active");

        decl.add_method(
            sel!(initWithServer:delegate:client:),
            init_with_server_delegate_client as extern "C" fn(&Object, Sel, id, id, id) -> id,
        );

        decl.add_method(
            sel!(activateServer:),
            activate_server as extern "C" fn(&mut Object, Sel, id),
        );

        decl.add_method(
            sel!(deactivateServer:),
            deactivate_server as extern "C" fn(&mut Object, Sel, id),
        );

        decl.add_method(
            sel!(handleCursorPositionRequest:),
            handle_cursor_position_request as extern "C" fn(&Object, Sel, id),
        );

        decl.add_method(
            sel!(respondsToSelector:),
            responds_to_selector as extern "C" fn(&Object, Sel, Sel) -> BOOL,
        );
    }
    decl.register();
    info!("finished registering {input_controller_class}.");
}

extern "C" fn init_with_server_delegate_client(this: &Object, _cmd: Sel, server: id, delegate: id, client: id) -> id {
    unsafe {
        info!("INITING");
        // Superclass
        let super_cls = Class::get("IMKInputController").unwrap();
        let this: id = msg_send![super(this, super_cls), initWithServer:server delegate: delegate
client: client];

        (*this).set_ivar::<BOOL>("is_active", NO);

        let mut center = NotificationCenter::distributed();
        center.subscribe_with_observer("io.fig.edit_buffer_updated", this, sel!(handleCursorPositionRequest:));

        this
    }
}

fn bundle_identifier(client: id) -> Option<String> {
    let bundle_id: NSStringRef = unsafe { msg_send![client, bundleIdentifier] };
    bundle_id.as_str().map(|s| s.into())
}

extern "C" fn activate_server(this: &mut Object, _cmd: Sel, client: id) {
    unsafe {
        (*this).set_ivar::<BOOL>("is_active", YES);
        info!("activated server: {:?}", bundle_identifier(client));
    }
}

extern "C" fn deactivate_server(this: &mut Object, _cmd: Sel, client: id) {
    unsafe {
        (*this).set_ivar::<BOOL>("is_active", NO);
        info!("deactivated server: {:?}", bundle_identifier(client));
    }
}

extern "C" fn handle_cursor_position_request(this: &Object, _sel: Sel, _notif: id) {
    let client: id = unsafe { msg_send![this, client] };
    let bundle_id = bundle_identifier(client);
    let is_active = unsafe { this.get_ivar::<BOOL>("is_active") };

    // Need to cast for some reason?
    if (*is_active) as i8 == 1 {
        info!("Instance {bundle_id:?} is active, handling request");
        let mut rect: NSRect = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: NSSize {
                height: 0.0,
                width: 0.0,
            },
        };

        let _: () = unsafe { msg_send![client, attributesForCharacterIndex: 0 lineHeightRectangle: &mut rect] };
        let max_y = unsafe {
            info!("Getting nsscreen");
            let screen: id = msg_send![class!(NSScreen), mainScreen];
            info!("Getting frame");
            if screen != cocoa::base::nil {
                let screen_bounds: NSRect = msg_send![screen, frame];
                Some(screen_bounds.origin.y + screen_bounds.size.height)
            } else {
                None
            }
        };
        info!("Got maxy: {max_y:?}");

        let hook = new_cursor_position_hook(
            rect.origin.x as i32,
            (max_y
                .map(|max_y| max_y - rect.origin.y - rect.size.height)
                .unwrap_or(0.0)) as i32,
            rect.size.width as i32,
            rect.size.height as i32,
        );

        info!("Sending cursor position for {bundle_id:?}: {hook:?}");
        tokio::spawn(async {
            match send_hook_to_socket(hook).await {
                Ok(_) => debug!("Sent hook successfully"),
                Err(_) => warn!("Failed to send hook"),
            }
        });
    }
}

extern "C" fn responds_to_selector(this: &Object, _cmd: Sel, selector: Sel) -> BOOL {
    info!("responds_to_selector");
    unsafe {
        info!("superclass");
        let superclass = msg_send![this, superclass];
        info!("should_respond");
        let should_respond: BOOL = msg_send![super(this, superclass), respondsToSelector: selector];
        info!("selector_name");
        let selector_name = CStr::from_ptr(sel_getName(selector))
            .to_str()
            .unwrap_or("UNKNOWN SELECTOR");
        trace!("`{}` should respond? {}", selector_name, should_respond);
        should_respond
    }
}
