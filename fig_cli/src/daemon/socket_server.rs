use std::net::{
    Ipv4Addr,
    Ipv6Addr,
    SocketAddrV4,
    SocketAddrV6,
};

use anyhow::{
    Context,
    Result,
};
use futures::{
    future,
    StreamExt,
    TryStreamExt,
};
use tokio::net::{
    TcpListener,
    TcpStream,
    ToSocketAddrs,
};
use tracing::{
    error,
    info,
};

pub type Port = u16;

// Discord uses range 6463 - 6472 for all devices. If you inspect /etc/services,
// you'll find that most ports leading to 6463 and following 6472 are reserved, but
// range 6463 - 6472 isn't. This is a range that was similar - non-reserved but
// surrounded by reserved port numbers.
const TCP_START_PORT: Port = 7920;
const TCP_END_PORT: Port = 7930;

async fn test_bind_tcp<A: ToSocketAddrs>(addr: A) -> Option<Port> {
    Some(TcpListener::bind(addr).await.ok()?.local_addr().ok()?.port())
}

async fn is_free_tcp(port: Port) -> bool {
    let ipv4 = SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port);
    let ipv6 = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0);

    test_bind_tcp(ipv6).await.is_some() && test_bind_tcp(ipv4).await.is_some()
}

async fn pick_unused_port(start: Port, end: Port) -> Option<Port> {
    for p in start..end {
        if is_free_tcp(p).await {
            return Some(p);
        }
    }
    None
}

pub async fn spawn_socket() -> Result<()> {
    let port = pick_unused_port(TCP_START_PORT, TCP_END_PORT)
        .await
        .context("Could not find free port")?;
    let addr = format!("127.0.0.1:{port}");

    // Create the event loop and TCP listener we'll accept connections on.
    let listener = TcpListener::bind(&addr).await?;

    info!("Listening on: {addr}");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(err) => {
            error!("Error getting peer address: {err}");
            return;
        },
    };
    info!("Peer address: {addr}");

    match tokio_tungstenite::accept_async(stream).await {
        Ok(ws_stream) => {
            info!("New WebSocket connection: {addr}");

            let (write, read) = ws_stream.split();
            // We should not forward messages other than text or binary.
            match read
                .try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
                .forward(write)
                .await
            {
                Ok(_) => {},
                Err(err) => {
                    error!("Error forwarding WebSocket: {err}");
                },
            }
        },
        Err(err) => error!("Error during the websocket handshake occurred: {err}"),
    }
}
