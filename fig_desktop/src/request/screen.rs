use std::sync::mpsc::{
    sync_channel,
    SyncSender,
};

use fig_proto::fig::{
    GetScreenshotRequest,
    GetScreenshotResponse,
    OpenContextMenuRequest,
};
use objc_foundation::INSData;
use screencapturekit::cm_sample_buffer::CMSampleBuffer;
use screencapturekit::sc_content_filter::{
    InitParams,
    SCContentFilter,
};
use screencapturekit::sc_error_handler::StreamErrorHandler;
use screencapturekit::sc_output_handler::{
    SCStreamOutputType,
    StreamOutput,
};
use screencapturekit::sc_shareable_content::SCShareableContent;
use screencapturekit::sc_stream::SCStream;
use screencapturekit::sc_stream_configuration::SCStreamConfiguration;
use screencapturekit::sc_types::SCFrameStatus;
use tracing::debug;

use super::{
    RequestResult,
    RequestResultImpl,
};
use crate::event::{
    Event,
    WindowEvent,
};
use crate::request::ServerOriginatedSubMessage;
use crate::webview::WindowId;
use crate::EventLoopProxy;

struct StoreImageHandler {
    tx: SyncSender<CMSampleBuffer>,
}

struct ErrorHandler;

impl StreamErrorHandler for ErrorHandler {
    fn on_error(&self) {
        eprintln!("ERROR!");
    }
}

impl StreamOutput for StoreImageHandler {
    fn did_output_sample_buffer(&self, sample: CMSampleBuffer, _of_type: SCStreamOutputType) {
        if let SCFrameStatus::Complete = sample.frame_status {
            self.tx.send(sample).ok();
        }
    }
}

pub fn take_screenshot(params: InitParams, width: u32, height: u32) -> Vec<u8> {
    let filter = SCContentFilter::new(params);

    let stream_config = SCStreamConfiguration {
        width,
        height,
        ..Default::default()
    };

    let (tx, rx) = sync_channel(2);
    let mut stream = SCStream::new(filter, stream_config, ErrorHandler);

    stream.add_output(StoreImageHandler { tx }, SCStreamOutputType::Screen);

    let _ = stream.start_capture();
    let sample_buf = rx.recv();

    stream.stop_capture().ok();

    if let Ok(buf) = sample_buf {
        if let Some(image) = buf.image_buf_ref {
            let jpeg = image.get_jpeg_data();
            jpeg.bytes().to_vec()
        } else {
            eprintln!("Screenshot could not be taken");
            vec![]
        }
    } else {
        eprintln!("Screenshot could not be taken");
        vec![]
    }
}

pub fn get_window_params(target: &str) -> Option<(InitParams, u32, u32)> {
    if let Some(window) = SCShareableContent::current()
        .windows
        .into_iter()
        .find(|window| window.title.as_ref().unwrap_or(&"".to_string()) == target)
    {
        let (width, height) = (window.width, window.height);
        let params = InitParams::DesktopIndependentWindow(window);
        Some((params, width, height))
    } else {
        None
    }
}

pub fn get_screenshot(request: GetScreenshotRequest, window_id: WindowId) -> RequestResult {
    debug!(?request, %window_id, "Get Screenshot Request");

    let mut images: Vec<Vec<u8>> = vec![];
    let target = request.target.as_str();

    if target == "ENTIRE" {
        for display in SCShareableContent::current().displays {
            let (width, height) = (display.width, display.height);
            let params = InitParams::Display(display);
            images.push(take_screenshot(params, width, height));
        }
    } else if let Some((params, width, height)) = get_window_params(target) {
        images.push(take_screenshot(params, width, height));
    }

    RequestResult::Ok(Box::new(ServerOriginatedSubMessage::GetScreenshotResponse(
        GetScreenshotResponse { images },
    )))
}

pub fn get_window_names() -> Vec<(String, String)> {
    let restricted_windows = ["", "Item-0"];
    let restricted_apps = ["", "Control Center", "Spotlight", "Dock", "Wallpaper"];
    let mut res: Vec<(String, String)> = vec![];

    for window in SCShareableContent::current().windows {
        if window.is_on_screen {
            if let (Some(title), Some(app)) = (window.title, window.owning_application) {
                if let Some(name) = app.application_name {
                    if !restricted_windows.contains(&title.as_str()) && !restricted_apps.contains(&name.as_str()) {
                        res.push((title, name));
                    }
                }
            }
        }
    }

    res
}

pub fn open_context_menu(
    request: OpenContextMenuRequest,
    window_id: WindowId,
    proxy: &EventLoopProxy,
) -> RequestResult {
    debug!(?request, %window_id, "Open Context Menu Request");

    if let Some(position) = request.position {
        proxy
            .send_event(Event::WindowEvent {
                window_id,
                window_event: WindowEvent::OpenContextMenu {
                    x: position.x,
                    y: position.y,
                    windows: get_window_names(),
                },
            })
            .unwrap();
    }

    RequestResult::success()
}
