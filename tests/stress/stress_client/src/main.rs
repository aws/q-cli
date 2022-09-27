use std::process::exit;
use std::time::Duration;

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
    build_serverbound,
    clientbound,
    Clientbound,
    Serverbound,
};
use tokio::net::UnixStream;
use tracing::{
    error,
    info,
};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let _guard = fig_log::Logger::new().with_stdout().init()?;

    let socket_path = std::env::var("STRESS_SOCKET")?;
    let kind = match std::env::var("STRESS_KIND")?.as_str() {
        "increment" => StressKind::Increment,
        "echo" => StressKind::Echo,
        _ => eyre::bail!("Invalid stress kind"),
    };
    let cycles: u64 = std::env::var("STRESS_CYCLES")?.parse()?;
    let sleep_for: u64 = std::env::var("STRESS_SLEEP")?.parse()?;
    let sleep_for = Duration::from_micros(sleep_for);
    let parallel: u64 = std::env::var("STRESS_PARALLEL")?.parse()?;
    let size: u64 = std::env::var("STRESS_SIZE")?.parse()?;

    let mut tasks = Vec::new();
    for i in 0..parallel {
        tasks.push(tokio::spawn(perform(
            i,
            socket_path.clone(),
            kind,
            cycles,
            sleep_for,
            size,
        )));
    }

    let mut results = Vec::new();
    for task in tasks {
        results.push(task.await??);
    }

    let mut success = true;
    for (i, result) in results.iter().enumerate() {
        if let Some(message) = result {
            error!("[{i}] test {kind:?} failed: {message}");
            success = false;
        }
    }

    if !success {
        exit(1);
    }

    Ok(())
}

async fn perform(
    i: u64,
    socket_path: String,
    kind: StressKind,
    cycles: u64,
    sleep_for: Duration,
    size: u64,
) -> eyre::Result<Option<String>> {
    info!("[{i}] connecting to server");
    let mut stream = BufferedReader::new(UnixStream::connect(socket_path).await?);

    let response: Clientbound = stream
        .send_recv_message_timeout(
            Serverbound {
                inner: Some(serverbound::Inner::Setup(serverbound::Setup {
                    kind: kind.into(),
                    i: i as i64,
                    cycles: cycles as i64,
                    payload_size: size as i64,
                })),
            },
            Duration::from_secs(10),
        )
        .await?
        .context("didn't receive ready packet")?;

    if response.inner.unwrap() != clientbound::Inner::Ready(()) {
        eyre::bail!("server didn't send ready packet");
    }

    info!("[{i}] ready to run test {kind:?}");

    match kind {
        StressKind::Increment => run_test_increment(&mut stream, cycles, sleep_for).await?,
        StressKind::Echo => run_test_echo(&mut stream, cycles).await?,
    }

    info!("[{i}] test complete, waiting for report");

    let report: Clientbound = stream
        .send_recv_message_timeout(
            Serverbound {
                inner: Some(serverbound::Inner::RequestReport(())),
            },
            Duration::from_secs(10),
        )
        .await?
        .context("didn't receive report")?;

    if let clientbound::Inner::Report(report) = report.inner.unwrap() {
        Ok(report.message)
    } else {
        eyre::bail!("received non-report message");
    }
}

async fn run_test_increment(
    stream: &mut BufferedReader<UnixStream>,
    cycles: u64,
    sleep_for: Duration,
) -> eyre::Result<()> {
    for i in 0..cycles {
        stream
            .send_message(build_serverbound(serverbound::Inner::IncrementTest(
                serverbound::IncrementTest { number: i as i64 },
            )))
            .await?;
        if !sleep_for.is_zero() {
            if sleep_for > Duration::from_millis(10) {
                tokio::time::sleep(sleep_for).await;
            } else {
                std::thread::sleep(sleep_for);
            }
        }
    }

    Ok(())
}

async fn run_test_echo(stream: &mut BufferedReader<UnixStream>, cycles: u64) -> eyre::Result<()> {
    for _ in 0..cycles {
        let recv: Clientbound = stream.recv_message().await?.unwrap();

        if let Some(clientbound::Inner::EchoTest(echo)) = recv.inner {
            stream
                .send_message(build_serverbound(serverbound::Inner::EchoResponse(
                    serverbound::EchoResponse { payload: echo.payload },
                )))
                .await?;
        } else {
            eyre::bail!("received invalid echo request");
        }
    }

    Ok(())
}
