#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use fig_proto::{
    figterm::*,
    local::{hook::Hook, local_message::Type, LocalMessage},
    FigMessage, FigProtobufEncodable,
};
use prost::Message;
use std::io::Cursor;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::vec::Vec;
use tauri::Window;
use windows::core::{Error, PCSTR, PSTR};
use windows::Win32::Foundation::{BOOL, BSTR, CHAR, HWND, RECT};
use windows::Win32::Networking::WinSock;
use windows::Win32::Storage::FileSystem;
use windows::Win32::System::Com::{
    self, CLSCTX_INPROC_SERVER, VARIANT, VARIANT_0, VARIANT_0_0, VARIANT_0_0_0,
};
use windows::Win32::System::Ole::VT_BSTR;
use windows::Win32::UI::Accessibility::{
    CUIAutomation8, IUIAutomation, IUIAutomationElement, TreeScope_Descendants, UIA_NamePropertyId,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId,
};

const WINSOCK_VERSION: u16 = 0x0202; // Windows Sockets version 2.2

/// contains rectangle bounds for Windows UI item.
#[derive(serde::Serialize, Default, PartialEq, Clone, Debug)]
struct UIRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl From<RECT> for UIRect {
    fn from(rect: RECT) -> Self {
        UIRect {
            x: rect.left,
            y: rect.top,
            w: rect.right - rect.left,
            h: rect.bottom - rect.top,
        }
    }
}

/// contains fields continually polled from foreground window.
#[derive(serde::Serialize, Default, PartialEq, Clone, Debug)]
struct WindowInfo {
    window_id: u32,
    process_id: u32,
    caret_pos: UIRect,
    window_pos: UIRect,
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            window_stream,
            socket_listener,
            insert_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// convert string path to array accepted by sockaddr_un struct
unsafe fn socket_path_to_arr(socket_path: &str) -> [CHAR; 108] {
    let path_char_vec: Vec<char> = String::from(socket_path).chars().collect();
    let mut path_char_arr: [CHAR; 108] = std::mem::zeroed();
    for i in 0..path_char_vec.len() {
        path_char_arr[i] = CHAR(path_char_vec[i] as u8);
    }
    path_char_arr
}

#[tauri::command]
fn socket_listener(window: Window) {
    println!("socket listener init");

    // as defined in rust-lib/fig-ipc
    // TODO: import from fig-ipc
    let fig_socket_path = r"C:\fig\fig.socket";

    std::thread::spawn(move || loop {
        unsafe {
            FileSystem::DeleteFileA(PCSTR(format!("{}\0", fig_socket_path).as_ptr()));

            // Windows socket startup
            let mut wsa_data: WinSock::WSAData = std::mem::zeroed();
            let mut ret: i32 =
                WinSock::WSAStartup(WINSOCK_VERSION, &mut wsa_data as *mut WinSock::WSAData);
            if ret != 0 {
                eprintln!("WSAStartup Error: {}", ret);
                return;
            }

            // create socket listener
            let listen_socket =
                WinSock::socket(WinSock::AF_UNIX.into(), WinSock::SOCK_STREAM.into(), 0);
            if listen_socket == WinSock::INVALID_SOCKET {
                eprintln!(
                    "Socket Initialization Error: {:?}",
                    WinSock::WSAGetLastError()
                );
                return;
            }

            // construct unix socket address
            let listener_addr: WinSock::sockaddr_un = WinSock::sockaddr_un {
                sun_family: WinSock::AF_UNIX,
                sun_path: socket_path_to_arr(fig_socket_path),
            };

            // bind socket to address
            // NOTE: transmute required as bind requires SOCKADDR ptr which only has buffer space
            // for 14 bytes (for use with IP addresses). sockaddr_un is meant for unix socket paths
            // and is allocated up to 108 bytes.
            ret = WinSock::bind(
                listen_socket,
                std::mem::transmute::<*const WinSock::sockaddr_un, *const WinSock::SOCKADDR>(
                    &listener_addr,
                ),
                std::mem::size_of::<WinSock::sockaddr_un>()
                    .try_into()
                    .unwrap(),
            );
            if ret == WinSock::SOCKET_ERROR {
                eprintln!("Bind Error: {:?}", WinSock::WSAGetLastError());
                return;
            }

            // thread-safe handle to tauri window
            let ts_window = Arc::new(Mutex::new(window));
            loop {
                // listen on socket listener
                ret = WinSock::listen(listen_socket, WinSock::SOMAXCONN.try_into().unwrap());
                if ret == WinSock::SOCKET_ERROR {
                    eprintln!("Socket Listen Error: {:?}", WinSock::WSAGetLastError());
                    return;
                }

                println!("Accepting connections on {}", fig_socket_path);

                // accept connections
                let mut addr: WinSock::SOCKADDR = std::mem::zeroed();
                let mut addrlen: i32 = std::mem::size_of::<WinSock::SOCKADDR>().try_into().unwrap();
                let client_socket = WinSock::accept(
                    listen_socket,
                    &mut addr as *mut WinSock::SOCKADDR,
                    &mut addrlen as *mut i32,
                );

                let window_handle = ts_window.clone();
                std::thread::spawn(move || {
                    if client_socket == WinSock::INVALID_SOCKET {
                        eprintln!("Invalid Socket: {:?}", WinSock::WSAGetLastError());
                        return;
                    }

                    println!("Accepted a connection.");

                    loop {
                        // read from socket
                        let mut rec_buffer: [u8; 1024] = std::mem::zeroed();
                        let res_result =
                            WinSock::recv(client_socket, PSTR(&mut rec_buffer as *mut u8), 1024, 0);
                        if res_result == WinSock::SOCKET_ERROR {
                            eprintln!("Socket Error: {:?}", WinSock::WSAGetLastError());
                            break;
                        }
                        if res_result == 0 {
                            println!("EOF received");
                            break;
                        }

                        println!("{} bytes received", res_result);

                        // parse fig message and send to frontend
                        let mut cursor = Cursor::new(&rec_buffer[..]);
                        match FigMessage::parse(&mut cursor) {
                            Ok(message) => {
                                let msg = protobuf_decode::<LocalMessage>(message);
                                if let Type::Hook(hook) = msg.clone().r#type.unwrap() {
                                    if let Hook::EditBuffer(edit_buffer) = hook.hook.unwrap() {
                                        let session_id =
                                            edit_buffer.context.unwrap().session_id.unwrap();
                                        let _res = window_handle
                                            .lock()
                                            .unwrap()
                                            .emit("session_id", session_id);
                                    }
                                }
                                let _res = window_handle
                                    .lock()
                                    .unwrap()
                                    .emit("figterm", String::from(format!("{:?}", msg)));
                            }
                            Err(err) => {
                                eprintln!("Error: {}", err);
                                break;
                            }
                        }
                    }

                    println!("shutting down");

                    // shutdown connection
                    ret = WinSock::shutdown(client_socket, 0);
                    if ret == WinSock::SOCKET_ERROR {
                        eprintln!("Shutdown Error: {:?}", WinSock::WSAGetLastError());
                        return;
                    }
                });
            }
        }
    });
}

#[tauri::command]
fn insert_text(session_id: String, text: String) {
    println!("inserting {} to  {}", text, session_id);

    // as defined in rust-lib/fig-ipc
    // TODO:import from fig-ipc
    let figterm_socket_path = format!(r"C:\fig\figterm-{}.socket", session_id);
    let fig_socket_path = r"C:\fig\fig.socket";

    unsafe {
        FileSystem::DeleteFileA(PCSTR(format!("{}\0", fig_socket_path).as_ptr()));

        // Windows socket startup
        let mut wsa_data: WinSock::WSAData = std::mem::zeroed();
        let mut ret: i32 =
            WinSock::WSAStartup(WINSOCK_VERSION, &mut wsa_data as *mut WinSock::WSAData);
        if ret != 0 {
            eprintln!("WSAStartup Error: {}", ret);
            return;
        }

        // create socket sender
        let client_socket =
            WinSock::socket(WinSock::AF_UNIX.into(), WinSock::SOCK_STREAM.into(), 0);
        if client_socket == WinSock::INVALID_SOCKET {
            eprintln!(
                "Socket Initialization Error: {:?}",
                WinSock::WSAGetLastError()
            );
            return;
        }

        // construct unix socket address
        let client_addr: WinSock::sockaddr_un = WinSock::sockaddr_un {
            sun_family: WinSock::AF_UNIX,
            sun_path: socket_path_to_arr(fig_socket_path),
        };

        // bind socket to address
        ret = WinSock::bind(
            client_socket,
            std::mem::transmute::<*const WinSock::sockaddr_un, *const WinSock::SOCKADDR>(
                &client_addr,
            ),
            std::mem::size_of::<WinSock::sockaddr_un>()
                .try_into()
                .unwrap(),
        );
        if ret == WinSock::SOCKET_ERROR {
            eprintln!("Bind Error: {:?}", WinSock::WSAGetLastError());
            return;
        }

        // construct unix server socket address
        let server_addr: WinSock::sockaddr_un = WinSock::sockaddr_un {
            sun_family: WinSock::AF_UNIX,
            sun_path: socket_path_to_arr(&figterm_socket_path),
        };

        // connect to figterm socket
        ret = WinSock::connect(
            client_socket,
            std::mem::transmute::<*const WinSock::sockaddr_un, *const WinSock::SOCKADDR>(
                &server_addr,
            ),
            std::mem::size_of::<WinSock::sockaddr_un>()
                .try_into()
                .unwrap(),
        );
        if ret == WinSock::SOCKET_ERROR {
            eprintln!("Socket Error: {:?}", WinSock::WSAGetLastError());
            return;
        }

        println!("Successfully connected");

        // construct command
        let cmd: InsertTextCommand = InsertTextCommand {
            insertion: Some(text),
            deletion: None,
            offset: None,
            immediate: None,
        };
        let figterm_msg: FigtermMessage = FigtermMessage {
            r#command: Some(figterm_message::Command::InsertTextCommand(cmd)),
        };

        // send command
        let res_result = WinSock::send(
            client_socket,
            PCSTR(figterm_msg.encode_fig_protobuf().unwrap().deref().as_ptr()),
            1024,
            WinSock::SEND_FLAGS(0),
        );
        if res_result == WinSock::SOCKET_ERROR {
            eprintln!("Send Error: {:?}", WinSock::WSAGetLastError());
            return;
        }

        println!("{} bytes sent", res_result);

        println!("Shutting down.");

        // shutdown connection
        ret = WinSock::shutdown(client_socket, 0);
        if ret == WinSock::SOCKET_ERROR {
            eprintln!("Shutdown Error: {:?}", WinSock::WSAGetLastError());
            return;
        }
    }
}

#[tauri::command]
fn window_stream(window: Window) {
    println!("window stream init");
    std::thread::spawn(move || {
        // cache to avoid spamming frontend with events
        let mut window_info_cache: WindowInfo = Default::default();
        loop {
            unsafe {
                // initialize Com library
                Com::CoInitialize(std::ptr::null()).unwrap();
                let hwnd: HWND = GetForegroundWindow();

                let window_id: u32 = hwnd.0 as u32;
                let process_id: u32 = get_process_id(hwnd);

                let new_window_info = WindowInfo {
                    window_id: window_id,
                    process_id: process_id,
                    caret_pos: UIRect::from(match get_caret_pos(hwnd) {
                        Ok(res) => res,
                        Err(_) => RECT::default(),
                    }),
                    window_pos: UIRect::from(match get_window_pos(hwnd) {
                        Some(res) => res,
                        None => RECT::default(),
                    }),
                };

                // send window info to frontend
                if window_info_cache != new_window_info {
                    window_info_cache = new_window_info.clone();
                    let _res = window.emit("wininfo", new_window_info);
                }

                Com::CoUninitialize();
            }
        }
    });
}

/// get pid of window process
unsafe fn get_process_id(hwnd: HWND) -> u32 {
    let mut pid: u32 = std::mem::zeroed();
    let _parent_pid = GetWindowThreadProcessId(hwnd, &mut pid as *mut u32);
    pid
}

/// get position of window
unsafe fn get_window_pos(hwnd: HWND) -> Option<RECT> {
    let mut win_pos: RECT = std::mem::zeroed();
    if GetWindowRect(hwnd, &mut win_pos as *mut RECT) == BOOL(0) {
        return None;
    }
    Some(win_pos)
}

/// get position of caret of window
unsafe fn get_caret_pos(hwnd: HWND) -> Result<RECT, Error> {
    let automation: IUIAutomation =
        Com::CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)?;

    let elt: IUIAutomationElement = automation.ElementFromHandle(hwnd)?;

    let terminal_input_pattern_condition =
        automation.CreatePropertyCondition(UIA_NamePropertyId, get_variant_caret_name())?;
    let caret_elt: IUIAutomationElement =
        elt.FindFirst(TreeScope_Descendants, terminal_input_pattern_condition)?;

    caret_elt.CurrentBoundingRectangle()
}

/// decode FigMessage
fn protobuf_decode<T>(message: FigMessage) -> T
where
    T: Message + Default,
{
    return T::decode(message.as_ref()).unwrap();
}

/// get object for searching for caret UI element by name
fn get_variant_caret_name() -> VARIANT {
    let shorts: &[u8] = "Terminal input".as_bytes();

    let mut longer: Vec<u16> = Vec::new();
    for &i in shorts {
        longer.push(i as u16);
    }

    VARIANT {
        Anonymous: VARIANT_0 {
            Anonymous: std::mem::ManuallyDrop::new(VARIANT_0_0 {
                vt: VT_BSTR.0 as u16,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: VARIANT_0_0_0 {
                    bstrVal: std::mem::ManuallyDrop::new(BSTR::from_wide(&longer[..])),
                },
            }),
        },
    }
}
