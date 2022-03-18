use clap::{Parser, Subcommand};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::remove_file;
use tokio::join;
use tokio::net::UnixListener;
use tokio::sync::Mutex;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    FigTerm(FigTermCommand),
    Listen,
    InputMethod,
    Demo,
}

#[derive(Parser)]
struct FigTermCommand {
    session_id: String,
    #[clap(subcommand)]
    message: FigTermMessage,
}

#[derive(Subcommand)]
enum FigTermMessage {
    InsertText {
        text: Option<String>,
        #[clap(short, long)]
        offset: Option<i64>,
        #[clap(short, long)]
        deletion: Option<u64>,
    },
}

#[tokio::main]
async fn main() {
    let args: Args = Args::parse();

    match args.action {
        Action::FigTerm(command) => send_message(command).await,
        Action::Listen => listen_for_messages().await,
        Action::InputMethod => listen_for_linux_messages().await,
        Action::Demo => demo().await,
    }
}

struct SharedState {
    text: String,
    idx: i64,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

async fn demo() {
    let app_socket_path = fig_ipc::get_fig_socket_path();
    let linux_socket_path = fig_ipc::get_fig_linux_socket_path();

    if app_socket_path.exists() {
        remove_file(&app_socket_path).await.unwrap();
    }

    if linux_socket_path.exists() {
        remove_file(&linux_socket_path).await.unwrap();
    }

    let shared_state = Arc::new(Mutex::new(SharedState {
        text: "".to_string(),
        idx: 0,
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    }));

    // buffer and idx data
    let app_state = shared_state.clone();
    let app_task = tokio::spawn(async move {
        let app_listener = UnixListener::bind(&app_socket_path).unwrap();
        loop {
            if let Ok((mut stream, _)) = app_listener.accept().await {
                let app_state = app_state.clone();
                loop {
                    use fig_proto::local::*;
                    match fig_ipc::recv_message::<fig_proto::local::LocalMessage, _>(&mut stream)
                        .await
                    {
                        // i love protobuf
                        Ok(Some(LocalMessage {
                            r#type:
                                Some(local_message::Type::Hook(Hook {
                                    hook:
                                        Some(hook::Hook::EditBuffer(EditBufferHook {
                                            text,
                                            cursor,
                                            ..
                                        })),
                                })),
                        })) => {
                            let mut handle = app_state.lock().await;
                            handle.text = text;
                            handle.idx = cursor;
                            drop(handle);
                            print_updated_info(&app_state).await;
                        }
                        Ok(None) => break,
                        Err(err) => {
                            println!("error receiving message: {:?}", err);
                            break;
                        }
                        _ => {}
                    }
                }
            };
        }
    });

    // cursor position data
    let linux_state = shared_state.clone();
    let linux_task = tokio::spawn(async move {
        let linux_listener = UnixListener::bind(&linux_socket_path).unwrap();
        loop {
            if let Ok((mut stream, _)) = linux_listener.accept().await {
                let linux_state = linux_state.clone();
                tokio::spawn(async move {
                    loop {
                        use fig_proto::linux::*;
                        match fig_ipc::recv_message::<fig_proto::linux::AppCommand, _>(&mut stream)
                            .await
                        {
                            // this one is a bit better
                            Ok(Some(AppCommand {
                                command:
                                    Some(app_command::Command::SetCursorPosition(
                                        SetCursorPositionCommand {
                                            x,
                                            y,
                                            width,
                                            height,
                                        },
                                    )),
                            })) => {
                                let mut handle = linux_state.lock().await;
                                handle.x = x;
                                handle.y = y;
                                handle.w = width;
                                handle.h = height;
                                drop(handle);
                                print_updated_info(&linux_state).await;
                            }
                            Ok(None) => break,
                            Err(err) => {
                                println!("error receiving message: {:?}", err);
                                break;
                            }
                            _ => {}
                        }
                    }
                });
            }
        }
    });

    let res = join!(app_task, linux_task);
    res.0.unwrap();
    res.1.unwrap();
}

async fn print_updated_info(state: &Arc<Mutex<SharedState>>) {
    let handle = state.lock().await;
    println!(
        "text: {} idx: {} x: {} y: {} w: {} h: {}",
        handle.text, handle.idx, handle.x, handle.y, handle.w, handle.h
    );
}

async fn send_message(args: FigTermCommand) {
    let socket_path = fig_ipc::figterm::get_figterm_socket_path(args.session_id.clone());
    println!("socket to connect to: {}", socket_path.to_string_lossy());

    use fig_proto::figterm as figterm_proto;
    let message = figterm_proto::FigtermMessage {
        command: Some(match args.message {
            FigTermMessage::InsertText {
                text,
                offset,
                deletion,
            } => figterm_proto::figterm_message::Command::InsertTextCommand(
                figterm_proto::InsertTextCommand {
                    insertion: text,
                    deletion,
                    offset,
                    immediate: Some(false),
                },
            ),
        }),
    };

    match fig_ipc::connect_timeout(socket_path, Duration::from_secs(1)).await {
        Ok(mut stream) => {
            if let Err(err) = fig_ipc::send_message(&mut stream, message).await {
                println!("error sending ipc message: {}", err);
            }
        }
        Err(err) => {
            println!("error connecting to socket: {}", err);
        }
    }
}

async fn listen_for_messages() {
    let socket_path = fig_ipc::get_fig_socket_path();

    if socket_path.exists() {
        remove_file(&socket_path).await.unwrap();
    }

    let socket_listener = UnixListener::bind(&socket_path).unwrap();

    loop {
        if let Ok((mut stream, _)) = socket_listener.accept().await {
            tokio::spawn(async move {
                loop {
                    match fig_ipc::recv_message::<fig_proto::local::LocalMessage, _>(&mut stream)
                        .await
                    {
                        Ok(Some(message)) => {
                            println!("new message: {:?}", message);
                        }
                        Ok(None) => break,
                        Err(err) => {
                            println!("error receiving message: {:?}", err);
                            break;
                        }
                    }
                }
            });
        }
    }
}

async fn listen_for_linux_messages() {
    let socket_path = fig_ipc::get_fig_linux_socket_path();

    if socket_path.exists() {
        remove_file(&socket_path).await.unwrap();
    }

    let socket_listener = UnixListener::bind(&socket_path).unwrap();

    loop {
        if let Ok((mut stream, _)) = socket_listener.accept().await {
            tokio::spawn(async move {
                loop {
                    match fig_ipc::recv_message::<fig_proto::linux::AppCommand, _>(&mut stream)
                        .await
                    {
                        Ok(Some(command)) => {
                            use fig_proto::linux::*;
                            if let Some(app_command::Command::SetCursorPosition(
                                set_cursor_position,
                            )) = command.command
                            {
                                println!(
                                    "set cursor position to {} {} {} {}",
                                    set_cursor_position.x,
                                    set_cursor_position.y,
                                    set_cursor_position.width,
                                    set_cursor_position.height
                                );
                            }
                        }
                        Ok(None) => break,
                        Err(err) => {
                            println!("error receiving message: {:?}", err);
                            break;
                        }
                    }
                }
            });
        }
    }
}
