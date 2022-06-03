use fig_proto::fig::OpenInExternalApplicationRequest;
use fig_util::open_url_async;

use super::{
    RequestResult,
    RequestResultImpl,
};

pub async fn open_in_external_application(request: OpenInExternalApplicationRequest) -> RequestResult {
    match request.url {
        Some(url) => match open_url_async(url).await {
            Ok(_) => RequestResult::success(),
            Err(err) => RequestResult::error(format!("Failed to open url: {err}")),
        },
        None => RequestResult::error("No url provided to open"),
    }
}
