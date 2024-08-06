use clap::Parser;
use fig_desktop_api::handler::{
    EventHandler,
    Wrapped,
};
use fig_desktop_api::kv::{
    DashKVStore,
    KVStore,
};
use fig_desktop_api::requests::{
    RequestResult,
    RequestResultImpl,
};
use fig_os_shim::{
    Env,
    EnvProvider,
    Fs,
    FsProvider,
};
use fig_proto::fig::NotificationRequest;

#[derive(Parser)]
enum Cli {
    Request {
        request_b64: String,
        #[arg(long)]
        cwd: Option<String>,
    },
    Init,
}

struct MockHandler;

struct Context {
    kv: DashKVStore,
    env: Env,
    fs: Fs,
}

impl KVStore for Context {
    fn set_raw(&self, key: impl Into<Vec<u8>>, value: impl Into<Vec<u8>>) {
        self.kv.set_raw(key, value);
    }

    fn get_raw(&self, key: impl AsRef<[u8]>) -> Option<Vec<u8>> {
        self.kv.get_raw(key)
    }
}

impl EnvProvider for Context {
    fn env(&self) -> &Env {
        &self.env
    }
}

impl FsProvider for Context {
    fn fs(&self) -> &Fs {
        &self.fs
    }
}

#[async_trait::async_trait]
impl EventHandler for MockHandler {
    type Ctx = Context;

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
            let response = fig_desktop_api::handler::api_request(
                MockHandler,
                Context {
                    kv: DashKVStore::new(),
                    env: Env::new(),
                    fs: Fs::new(),
                },
                request,
            )
            .await
            .unwrap();
            let response_b64 = fig_desktop_api::handler::response_to_b64(response);
            println!("{response_b64}");
        },
        Cli::Init => {
            println!("{}", fig_desktop_api::init_script::javascript_init(false));
        },
    }
}
