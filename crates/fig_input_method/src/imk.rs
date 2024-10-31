use std::ffi::CStr;

use cocoa::base::{
    BOOL,
    NO,
    YES,
    id,
};
use cocoa::foundation::{
    NSPoint,
    NSRange,
    NSRect,
    NSSize,
};
use fig_ipc::local::send_hook_to_socket;
use fig_proto::hooks::new_caret_position_hook;
use fig_proto::local::caret_position_hook::Origin;
use fig_util::Terminal;
use macos_utils::{
    NSString,
    NSStringRef,
    NotificationCenter,
};
use objc::declare::ClassDecl;
use objc::runtime::{
    Class,
    Object,
    Sel,
    sel_getName,
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

        let mut center = NotificationCenter::distributed_center();
        center.subscribe_with_observer(
            "com.amazon.codewhisperer.edit_buffer_updated",
            this,
            sel!(handleCursorPositionRequest:),
        );

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
        let bundle_id = bundle_identifier(client);
        info!("activated server: {:?}", bundle_id);

        let terminal = Terminal::from_bundle_id(bundle_id.unwrap_or_default().as_str());

        // Used to trigger input method enabled in Alacritty
        if matches!(terminal, Some(Terminal::Alacritty)) {
            let empty_range = NSRange::new(0, 0);
            let space_string: NSString = " ".into();
            let empty_string: NSString = "".into();

            // First, setMarkedText with a non-empty string in order to enable winit IME
            // https://github.com/rust-windowing/winit/blob/97d4c7b303bb8110df6c492f0c2327b7d5098347/src/platform_impl/macos/view.rs#L330-L337

            let _: () = msg_send![client, setMarkedText: space_string selectionRange: empty_range replacementRange: empty_range];

            // Then, since we don't *actually* want to be in the preedit state, set marked text to an empty
            // string to invalidate https://github.com/rust-windowing/winit/blob/97d4c7b303bb8110df6c492f0c2327b7d5098347/src/platform_impl/macos/view.rs#L345-L351
            let _: () = msg_send![client, setMarkedText: empty_string selectionRange: empty_range replacementRange: empty_range];
        }
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

    if *is_active == YES {
        let terminal = Terminal::from_bundle_id(bundle_id.as_deref().unwrap_or_default());
        match terminal {
            Some(term) if term.supports_macos_input_method() => {
                info!("Instance {bundle_id:?} is active, handling request");
                let mut rect: NSRect = NSRect {
                    origin: NSPoint { x: 0.0, y: 0.0 },
                    size: NSSize {
                        height: 0.0,
                        width: 0.0,
                    },
                };
                let _: () = unsafe { msg_send![client, attributesForCharacterIndex: 0 lineHeightRectangle: &mut rect] };

                let hook = new_caret_position_hook(
                    rect.origin.x,
                    rect.origin.y,
                    rect.size.width,
                    rect.size.height,
                    Origin::BottomLeft,
                );

                info!("Sending cursor position for {bundle_id:?}: {hook:?}");
                tokio::spawn(async {
                    match send_hook_to_socket(hook).await {
                        Ok(_) => debug!("Sent hook successfully"),
                        Err(_) => warn!("Failed to send hook"),
                    }
                });
            },
            _ => {
                info!("Instance {bundle_id:?} is not a supported terminal, ignoring request");
            },
        }
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
