use clap::Parser;
use fig_desktop_api::handler::{
    EventHandler,
    Wrapped,
};
use fig_desktop_api::requests::{
    RequestResult,
    RequestResultImpl,
};
use fig_proto::fig::NotificationRequest;

#[derive(Parser)]
enum Cli {
    Request {
        request_b64: String,
        #[clap(long)]
        cwd: Option<String>,
    },
    Init,
}

struct MockHandler;

#[async_trait::async_trait]
impl EventHandler for MockHandler {
    type Ctx = ();

    async fn notification(&self, _request: Wrapped<Self::Ctx, NotificationRequest>) -> RequestResult {
        RequestResult::success()
    }
}

#[tokio::main]
async fn main() {
    match Cli::parse() {
        Cli::Request { request_b64, cwd } => {
            if let Some(cwd) = cwd {
                std::env::set_current_dir(cwd).unwrap();
            }

            let request = fig_desktop_api::handler::request_from_b64(&request_b64).unwrap();
            let response = fig_desktop_api::handler::api_request(MockHandler, (), request)
                .await
                .unwrap();
            let response_b64 = fig_desktop_api::handler::response_to_b64(response);
            println!("{response_b64}");
        },
        Cli::Init => {
            println!("{}", fig_desktop_api::init_script::javascript_init());
        },
    }
}
