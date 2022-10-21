#[cfg(not(target_os = "linux"))]
pub async fn check_for_update(show_webview: bool) -> bool {
    use fig_install::UpdateStatus;
    use tokio::sync::mpsc::Receiver;
    use wry::application::dpi::LogicalSize;
    use wry::application::menu::{
        MenuBar,
        MenuItem,
    };
    use wry::application::platform::macos::WindowBuilderExtMacOS;

    use crate::utils::is_cargo_debug_build;

    let updating_cb: Option<Box<dyn FnOnce(Receiver<UpdateStatus>) + Send>> = if show_webview {
        Some(Box::new(|mut recv: Receiver<UpdateStatus>| {
            use wry::application::event::{
                Event,
                WindowEvent,
            };
            use wry::application::event_loop::{
                ControlFlow,
                EventLoop,
            };
            use wry::application::window::WindowBuilder;
            use wry::webview::WebViewBuilder;

            let mut menu_bar = MenuBar::new();
            let mut sub_menu_bar = MenuBar::new();
            sub_menu_bar.add_native_item(MenuItem::Quit);
            menu_bar.add_submenu("Fig", true, sub_menu_bar);

            let event_loop: EventLoop<UpdateStatus> = EventLoop::with_user_event();
            let window = WindowBuilder::new()
                .with_title("Fig")
                .with_inner_size(LogicalSize::new(350, 350))
                .with_resizable(false)
                .with_titlebar_hidden(true)
                .with_movable_by_window_background(true)
                .with_menu(menu_bar)
                .build(&event_loop)
                .unwrap();

            let webview = WebViewBuilder::new(window)
                .unwrap()
                .with_html(include_str!("../html/updating.html"))
                .unwrap()
                .with_devtools(true)
                .build()
                .unwrap();

            // Forward recv to the webview
            let proxy = event_loop.create_proxy();
            std::thread::spawn(move || {
                // Sleep for a little bit for the js to initialize (dont know why :()
                std::thread::sleep(std::time::Duration::from_millis(500));
                loop {
                    if let Some(event) = recv.blocking_recv() {
                        proxy.send_event(event).ok();
                    }
                }
            });

            event_loop.run(move |event, _, control_flow| {
                *control_flow = ControlFlow::Wait;

                match event {
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => *control_flow = ControlFlow::Exit,
                    Event::UserEvent(event) => match event {
                        UpdateStatus::Percent(p) => {
                            webview
                                .evaluate_script(&format!("updateProgress({});", p as i32))
                                .unwrap();
                        },
                        UpdateStatus::Message(message) => {
                            webview
                                .evaluate_script(&format!("updateMessage({});", serde_json::json!(message)))
                                .unwrap();
                        },
                        UpdateStatus::Error(message) => {
                            webview
                                .evaluate_script(&format!("updateError({});", serde_json::json!(message)))
                                .unwrap();
                        },
                        UpdateStatus::Exit => {
                            *control_flow = ControlFlow::Exit;
                        },
                    },
                    _ => {},
                }
            });
        }))
    } else {
        None
    };

    // If not debug or override, check for update
    if !is_cargo_debug_build() && fig_settings::settings::get_bool_or("app.disableAutoupdates", true) {
        match fig_install::update(true, updating_cb, !show_webview).await {
            Ok(status) => status,
            Err(err) => {
                tracing::error!(%err, "Failed to update");
                false
            },
        }
    } else {
        false
    }
}
