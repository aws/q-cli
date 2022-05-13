use core::pin::Pin;
use core::task::{
    Context,
    Poll,
};
use std::convert::TryInto;
use std::io;
use std::path::Path;

use tokio::io::{
    AsyncRead,
    AsyncWrite,
    ReadBuf,
};
use tracing::trace;
use windows::core::{
    PCSTR,
    PSTR,
};
use windows::Win32::Foundation::CHAR;
use windows::Win32::Networking::WinSock::{
    SEND_RECV_FLAGS,
    {self,},
};
use windows::Win32::Storage::FileSystem;

#[derive(Debug)]
pub enum WinSockError {
    StartupFailed,
    InitializationFailed(WinSock::WSA_ERROR),
    BindingFailed(WinSock::WSA_ERROR),
    ListeningFailed(WinSock::WSA_ERROR),
    InvalidSocket(WinSock::WSA_ERROR),
}

pub struct WindowsStream {
    socket: WinSock::SOCKET,
}

impl AsyncRead for WindowsStream {
    fn poll_read(self: Pin<&mut Self>, _cx: &mut Context<'_>, buf: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        let res = unsafe {
            WinSock::recv(
                self.socket,
                PSTR(&mut buf.initialize_unfilled_to(1024)[0] as *mut u8),
                1024,
                SEND_RECV_FLAGS(0),
            )
        };
        if res == WinSock::SOCKET_ERROR {
            return Poll::Ready(Err(io::Error::from(io::ErrorKind::TimedOut)));
        }
        buf.set_filled(res.try_into().unwrap());
        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for WindowsStream {
    fn poll_write(self: Pin<&mut Self>, _cx: &mut Context<'_>, _buf: &[u8]) -> Poll<Result<usize, io::Error>> {
        todo!()
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        todo!()
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        todo!()
    }
}

pub struct WindowsListener {
    listen_socket: WinSock::SOCKET,
}

impl WindowsListener {
    /// convert string path to array accepted by sockaddr_un struct
    fn socket_path_to_arr(socket_path: &Path) -> [CHAR; 108] {
        let path_char_vec: Vec<char> = socket_path.to_str().unwrap().chars().collect();
        let mut path_char_arr: [CHAR; 108] = [CHAR(0); 108];
        for i in 0..path_char_vec.len() {
            path_char_arr[i] = CHAR(path_char_vec[i] as u8);
        }
        path_char_arr
    }

    pub fn bind(socket_path: &Path) -> Result<Self, WinSockError> {
        const WINSOCK_VERSION: u16 = 0x0202; // Windows Socket version 2.2

        // TODO: Use tokio::fs
        unsafe {
            FileSystem::DeleteFileA(PCSTR(format!("{}\0", socket_path.to_str().unwrap()).as_ptr()));
        }

        // Windows socket startup
        let mut wsa_data: WinSock::WSAData = Default::default();
        let mut ret: i32 = unsafe { WinSock::WSAStartup(WINSOCK_VERSION, &mut wsa_data as *mut WinSock::WSAData) };
        if ret != 0 {
            return Err(WinSockError::StartupFailed);
        }

        // create socket listener
        let listen_socket = unsafe { WinSock::socket(WinSock::AF_UNIX.into(), WinSock::SOCK_STREAM.into(), 0) };
        if listen_socket == WinSock::INVALID_SOCKET {
            return unsafe { Err(WinSockError::InitializationFailed(WinSock::WSAGetLastError())) };
        }

        // construct unix socket address
        let listener_addr: WinSock::sockaddr_un = WinSock::sockaddr_un {
            sun_family: WinSock::AF_UNIX,
            sun_path: Self::socket_path_to_arr(socket_path),
        };

        // bind socket to address
        // NOTE: transmute required as bind requires SOCKADDR ptr which only has buffer space
        // for 14 bytes (for use with IP addresses). sockaddr_un is meant for unix socket paths
        // and is allocated up to 108 bytes.
        ret = unsafe {
            WinSock::bind(
                listen_socket,
                std::mem::transmute::<*const WinSock::sockaddr_un, *const WinSock::SOCKADDR>(&listener_addr),
                std::mem::size_of::<WinSock::sockaddr_un>().try_into().unwrap(),
            )
        };
        if ret == WinSock::SOCKET_ERROR {
            return unsafe { Err(WinSockError::BindingFailed(WinSock::WSAGetLastError())) };
        }

        Ok(Self { listen_socket })
    }

    pub async fn accept(&self) -> Result<WindowsStream, WinSockError> {
        // listen on socket listener
        let ret = unsafe { WinSock::listen(self.listen_socket, WinSock::SOMAXCONN.try_into().unwrap()) };
        if ret == WinSock::SOCKET_ERROR {
            return unsafe { Err(WinSockError::ListeningFailed(WinSock::WSAGetLastError())) };
        }

        trace!("Accepting connections");

        // accept connections
        let mut addr: WinSock::SOCKADDR = Default::default();
        let mut addrlen: i32 = std::mem::size_of::<WinSock::SOCKADDR>().try_into().unwrap();
        let client_socket = unsafe {
            WinSock::accept(
                self.listen_socket,
                &mut addr as *mut WinSock::SOCKADDR,
                &mut addrlen as *mut i32,
            )
        };

        if client_socket == WinSock::INVALID_SOCKET {
            return unsafe { Err(WinSockError::InvalidSocket(WinSock::WSAGetLastError())) };
        }

        Ok(WindowsStream { socket: client_socket })
    }
}
