#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

#[cfg(target_os = "macos")]
mod imk;

#[cfg(target_os = "macos")]
#[tokio::main]
async fn main() {
    use cocoa::appkit::{
        NSApp,
        NSApplication,
    };
    use cocoa::base::{
        id,
        nil,
        BOOL,
    };
    use cocoa::foundation::{
        NSAutoreleasePool,
        NSString,
    };
    use fig_log::{
        set_fig_log_level,
        Logger,
    };
    use tracing::info;

    let logger = Logger::new().with_file("imk.log").with_stdout();
    let _logger_guard = logger.init().expect("Failed to init logger");
    set_fig_log_level("trace".to_string()).ok();
    info!("HI THERE");

    imk::register_controller();

    info!("registered imk controller");

    let connection_name: &str = match option_env!("InputMethodConnectionName") {
        Some(name) => name,
        None => unreachable!("InputMethodConnectionName environment var must be specified"),
    };

    unsafe {
        let _pool = NSAutoreleasePool::new(nil);
        let app = NSApp();
        let k_connection_name = NSString::alloc(nil).init_str(connection_name);
        let nib_name = NSString::alloc(nil).init_str("MainMenu");

        let bundle: id = msg_send![class!(NSBundle), mainBundle];
        let identifier: id = msg_send![bundle, bundleIdentifier];

        info!("Attempting connection...");
        imk::connect_imkserver(k_connection_name, identifier);
        info!("Connected!");

        let loaded_nib: BOOL = msg_send![class!(NSBundle), loadNibNamed:nib_name
                                owner:app];
        info!("RUNNING {loaded_nib:?}!");
        app.run();
    }
}

#[cfg(not(target_os = "macos"))]
fn main() {
    println!("Fig input method is only supported on macOS");
}
