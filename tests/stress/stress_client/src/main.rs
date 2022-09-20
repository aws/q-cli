use std::process::exit;
use std::time::Duration;

use eyre::ContextCompat;
use fig_ipc::{
    BufferedReader,
    SendMessage,
    SendRecvMessage,
};
use fig_proto::stress::serverbound::{
    self,
    StressKind,
};
use fig_proto::stress::{
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
        _ => eyre::bail!("Invalid stress kind"),
    };
    let cycles: u64 = std::env::var("STRESS_CYCLES")?.parse()?;
    let sleep_for: u64 = std::env::var("STRESS_SLEEP")?.parse()?;
    let sleep_for = Duration::from_micros(sleep_for);
    let parallel: u64 = std::env::var("STRESS_PARALLEL")?.parse()?;

    let mut tasks = Vec::new();
    for i in 0..parallel {
        tasks.push(tokio::spawn(perform(i, socket_path.clone(), kind, cycles, sleep_for)));
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
) -> eyre::Result<Option<String>> {
    info!("[{i}] connecting to server");
    let mut stream = BufferedReader::new(UnixStream::connect(socket_path).await?);

    let response: Clientbound = stream
        .send_recv_message_timeout(
            Serverbound {
                inner: Some(serverbound::Inner::Setup(serverbound::Setup {
                    kind: StressKind::Increment.into(),
                    i: i as i64,
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
            .send_message(Serverbound {
                inner: Some(serverbound::Inner::IncrementTest(serverbound::IncrementTest {
                    number: i as i64,
                })),
            })
            .await?;
        if !sleep_for.is_zero() {
            std::thread::sleep(sleep_for);
        }
    }

    Ok(())
}
