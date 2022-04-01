use crate::{
    os::{native, GenericListener},
    state::AppStateType,
};

pub async fn start_local_ipc(_state: AppStateType) {
    let socket_path = fig_ipc::get_fig_socket_path();

    if socket_path.exists() {
        tokio::fs::remove_file(&socket_path)
            .await
            .expect("Failed clearing socket path");
    }

    let socket = native::bind_socket(&socket_path);

    while let Ok(_stream) = socket.generic_accept().await {
        todo!() // TODO(mia): finish this bit
    }
}
