use std::path::Path;

use eyre::ContextCompat;
use fig_ipc::{
    BufferedReader,
    RecvMessage,
    SendMessage,
    SendRecvMessage,
};
use fig_proto::stress::serverbound::{
    self,
    StressKind,
};
use fig_proto::stress::{
    build_clientbound,
    clientbound,
    Clientbound,
    Serverbound,
};
use tokio::net::{
    UnixListener,
    UnixStream,
};
use tracing::{
    error,
    info,
    warn,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = fig_log::Logger::new().with_stdout().init()?;

    let socket_path = std::env::var("STRESS_SOCKET")?;
    let close_after: u64 = std::env::var("STRESS_CLOSE_AFTER")?.parse()?;

    if Path::exists(socket_path.as_ref()) {
        std::fs::remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;
    let mut started_connections = 0;

    let mut tasks = Vec::new();

    info!("waiting for connections");

    while started_connections < close_after {
        let (stream, _) = listener.accept().await?;
        started_connections += 1;
        let stream = BufferedReader::new(stream);
        tasks.push(tokio::spawn(async move {
            if let Err(err) = handle_stream(stream).await {
                error!(?err, "error handling stream");
            }
        }));
    }

    info!("waiting for tasks to exit");

    for task in tasks {
        task.await?;
    }

    Ok(())
}

async fn handle_stream(mut stream: BufferedReader<UnixStream>) -> eyre::Result<()> {
    let setup: Serverbound = stream
        .recv_message()
        .await?
        .context("client didn't send setup packet")?;

    let (kind, i, cycles, payload_size) = if let serverbound::Inner::Setup(setup) = setup.inner.unwrap() {
        (
            setup.kind(),
            setup.i as u64,
            setup.cycles as u64,
            setup.payload_size as u64,
        )
    } else {
        eyre::bail!("client skipped setup packet");
    };

    let message = match kind {
        StressKind::Increment => run_test_increment(&mut stream, i).await?,
        StressKind::Echo => run_test_echo(&mut stream, i, cycles, payload_size).await?,
    };
    info!("[{i}] test complete, sending report");
    stream
        .send_message(Clientbound {
            inner: Some(clientbound::Inner::Report(clientbound::Report { message })),
        })
        .await?;

    Ok(())
}

async fn run_test_increment(stream: &mut BufferedReader<UnixStream>, i: u64) -> eyre::Result<Option<String>> {
    ready(stream, "increment", i).await?;

    let mut failure = None;

    let mut expected = 0;
    while let Some(message) = recv(stream).await? {
        if let serverbound::Inner::IncrementTest(serverbound::IncrementTest { number }) = message.inner.unwrap() {
            if number != expected {
                failure = Some(format!("number mismatch: got {number}, expected {expected}"));
                warn!("[{i}] increment test: failure {number} != {expected}");
            }
            expected += 1;
        }
    }

    Ok(failure)
}

async fn run_test_echo(
    stream: &mut BufferedReader<UnixStream>,
    i: u64,
    cycles: u64,
    payload_size: u64,
) -> eyre::Result<Option<String>> {
    ready(stream, "echo", i).await?;

    let mut failure = None;

    for _ in 0..cycles {
        let mut payload = vec![0u8; payload_size as usize];
        payload.fill_with(rand::random);
        let payload = hex::encode(payload);
        let echoed: Serverbound = stream
            .send_recv_message(build_clientbound(clientbound::Inner::EchoTest(clientbound::EchoTest {
                payload: payload.clone(),
            })))
            .await?
            .context("missing echo response")?;
        if let Some(serverbound::Inner::EchoResponse(echo)) = echoed.inner {
            if echo.payload != payload {
                failure = Some(format!("echo mismatch: got {}, expect {payload}", echo.payload));
            }
        } else {
            failure = Some("received invalid echo response".to_string());
        }
    }

    Ok(failure)
}

async fn ready(stream: &mut BufferedReader<UnixStream>, kind: &str, i: u64) -> eyre::Result<()> {
    stream
        .send_message(Clientbound {
            inner: Some(clientbound::Inner::Ready(())),
        })
        .await?;

    info!("[{i}] ready to accept test {kind}");

    Ok(())
}

fn check_report_request(message: Serverbound) -> Option<Serverbound> {
    if message.inner == Some(serverbound::Inner::RequestReport(())) {
        None
    } else {
        Some(message)
    }
}

async fn recv(stream: &mut BufferedReader<UnixStream>) -> eyre::Result<Option<Serverbound>> {
    Ok(stream.recv_message().await?.and_then(check_report_request))
}
