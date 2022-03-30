#![warn(clippy::pedantic)]
#![warn(rust_2018_idioms)]

use std::borrow::Cow;
use libsystemd::activation::IsType;
use std::fs;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::os::unix::net::UnixListener as StdUnixListener;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio::net::{UnixListener, UnixStream};

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let systemd_fds = libsystemd::activation::receive_descriptors(true);
    let socket = if let Ok(mut fds) = systemd_fds {
        let fd = fds
            .pop()
            .unwrap_or_else(|| panic!("no file descriptor passed from systemd"));
        if !fd.is_unix() {
            panic!("systemd passed file descriptor should be a unix socket");
        }
        // SAFETY: into_raw_fd moves the value and receive_descriptors unsets the env variable
        let listener = unsafe { StdUnixListener::from_raw_fd(fd.into_raw_fd()) };
        UnixListener::from_std(listener).unwrap()
    } else {
        let path = fig_ipc::get_fig_socket_path();
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }
        UnixListener::bind(path).unwrap()
    };
    println!(
        "starting daemon on socket {:?}",
        socket.local_addr().unwrap()
    );
    while let Ok((stream, _)) = socket.accept().await {
        tokio::spawn(handle(stream));
    }
    println!("daemon exiting");
}

enum ResponseKind {
    Error(String),
    Success,
    Message(fig_proto::fig::server_originated_message::Submessage),
}

async fn handle(mut stream: UnixStream) {
    use fig_proto::fig::*;
    while let Some(message) = fig_ipc::recv_message::<ClientOriginatedMessage, _>(&mut stream)
        .await
        .unwrap_or_else(|err| {
            println!("error receiving message: {:?}", err);
            None
        })
    {
        println!("message: {:?}", message);
        use client_originated_message::Submessage;
        let response = match message.submessage {
            Some(Submessage::ReadFileRequest(req)) => read_file(req).await,
            None => ResponseKind::Error("Command not specified".to_string()),
            _ => ResponseKind::Error("Command not supported".to_string()),
        };

        let message = ServerOriginatedMessage {
            id: message.id,
            submessage: Some(match response {
                ResponseKind::Error(msg) => server_originated_message::Submessage::Error(msg),
                ResponseKind::Success => server_originated_message::Submessage::Success(true),
                ResponseKind::Message(m) => m,
            })
        };

        fig_ipc::send_message(&mut stream, message).await;
    }
}

async fn read_file(req: fig_proto::fig::ReadFileRequest) -> ResponseKind {
    use fig_proto::fig::*;
    let file_path = match req.path {
        Some(s) => s,
        None => return ResponseKind::Error("Missing path".to_string())
    };
    let convert = |path: String| {
        if file_path.expand_tilde_in_path {
            shellexpand::tilde(&path).into_owned()
        } else {
            path
        }
    };
    let mut relative_to = file_path.relative_to.map(convert).map(PathBuf::from).unwrap_or_else(PathBuf::new);
    let path = PathBuf::from(convert(file_path.path));
    relative_to.push(path);
    let file_data = match tokio::fs::read(relative_to).await {
        Ok(s) => s,
        Err(err) => return ResponseKind::Error(format!("Failed reading file: {:?}", err)),
    };
    ResponseKind::Message(server_originated_message::Submessage::ReadFileResponse(ReadFileResponse {
        r#type: Some(if req.is_binary_file {
            read_file_response::Type::Data(file_data)
        } else {
            read_file_response::Type::Text(String::from_utf8_lossy(&file_data).to_string())
        })
    }))
}
