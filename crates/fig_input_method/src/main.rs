// This is needed for objc
#![allow(unexpected_cfgs)]

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
        BOOL,
        id,
        nil,
    };
    use cocoa::foundation::{
        NSAutoreleasePool,
        NSString,
    };
    use fig_log::{
        LogArgs,
        initialize_logging,
    };
    use fig_util::directories;
    use tracing::info;

    let _log_guard = initialize_logging(LogArgs {
        log_level: Some("trace".to_owned()),
        log_to_stdout: true,
        log_file_path: Some(directories::logs_dir().expect("home dir must be set").join("imk.log")),
        delete_old_log_file: false,
    });

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
