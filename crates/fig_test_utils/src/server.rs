use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use bytes::Bytes;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::Service;
use hyper::{
    Request,
    Response,
};
use hyper_util::rt::TokioIo;
use tokio::net::{
    TcpListener,
    TcpStream,
};
use tokio::select;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct TestServer {
    listener: TcpListener,
    cancellation_token: Option<CancellationToken>,
    mock_responses: HashMap<(http::Method, String), String>,
}

impl TestServer {
    pub async fn new() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("failed to bind socket to local host");
        Self {
            listener,
            cancellation_token: None,
            mock_responses: HashMap::default(),
        }
    }

    pub fn with_mock_response(mut self, method: http::Method, path: String, response: String) -> Self {
        self.mock_responses.insert((method, path), response);
        self
    }

    pub fn spawn_listener(mut self) -> SocketAddr {
        let addr = self
            .listener
            .local_addr()
            .expect("listener should be bound to an address");
        let token = CancellationToken::new();
        let token_clone = token.clone();
        self.cancellation_token = Some(token);
        tokio::task::spawn(async move {
            loop {
                select! {
                    stream = self.listener.accept() => {
                        let (stream, _) = stream.expect("failed to accept new connection");
                        self.handle_request(stream).await;
                    },
                    _ = token_clone.cancelled() => {
                        break;
                    }
                }
            }
        });
        addr
    }

    async fn handle_request(&self, stream: TcpStream) {
        let stream = TokioIo::new(stream);
        hyper::server::conn::http1::Builder::new()
            .serve_connection(stream, &self)
            .await
            .unwrap();
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        if let Some(token) = &self.cancellation_token {
            token.cancel();
        }
    }
}

type ServiceError = Box<dyn std::error::Error + Send + Sync + 'static>;
type ServiceResponse = Response<Full<Bytes>>;
type ServiceFuture = Pin<Box<dyn Future<Output = Result<ServiceResponse, ServiceError>> + Send>>;

impl Service<Request<Incoming>> for TestServer {
    type Error = ServiceError;
    type Future = ServiceFuture;
    type Response = ServiceResponse;

    fn call(&self, req: Request<Incoming>) -> Self::Future {
        let method = req.method().clone();
        let path = req.uri().path().to_string();
        let result = self.mock_responses.get(&(method, path)).unwrap().clone();
        Box::pin(async move { Ok(Response::builder().status(200).body(result.into()).unwrap()) })
    }
}
