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
use tauri::{State, Window};
use windows::core::{Error, PCSTR, PSTR};
use windows::Win32::Foundation::{BOOL, BSTR, CHAR, HWND, RECT};
use windows::Win32::Networking::WinSock;
use windows::Win32::Storage::FileSystem;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitialize, CoUninitialize, CLSCTX_INPROC_SERVER, VARIANT, VARIANT_0,
    VARIANT_0_0, VARIANT_0_0_0,
};
use windows::Win32::System::Ole::VT_BSTR;
use windows::Win32::UI::Accessibility::{
    CUIAutomation8, IUIAutomation, IUIAutomationElement, TreeScope_Descendants, UIA_NamePropertyId,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId,
};

#[derive(serde::Serialize, Default, PartialEq, Clone, Debug)]
struct WindowBound {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

impl From<RECT> for WindowBound {
    fn from(rect: RECT) -> Self {
        WindowBound {
            x: rect.left,
            y: rect.top,
            w: rect.right - rect.left,
            h: rect.bottom - rect.top,
        }
    }
}

#[derive(serde::Serialize, Default, PartialEq, Clone, Debug)]
struct WindowInfo {
    window_id: u32,
    process_id: u32,
    caret_pos: WindowBound,
    window_pos: WindowBound,
}

#[derive(Default)]
struct WindowInfoPayload(Arc<Mutex<WindowInfo>>);

fn main() {
    tauri::Builder::default()
        .manage(WindowInfoPayload(Default::default()))
        .invoke_handler(tauri::generate_handler![
            window_stream,
            socket_listener,
            insert_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn socket_listener(window: Window) {
    println!("socket listener init");

    std::thread::spawn(move || loop {
        unsafe {
            let FIG_SOCKET_PATH = r"C:\fig\fig.socket";

            FileSystem::DeleteFileA(PCSTR(format!("{}\0", FIG_SOCKET_PATH).as_ptr()));

            let mut ListenSocket: WinSock::SOCKET = WinSock::INVALID_SOCKET;
            let mut ClientSocket: WinSock::SOCKET = WinSock::INVALID_SOCKET;
            let mut wsa_data: WinSock::WSAData = std::mem::zeroed();

            let mut ret: i32 = WinSock::WSAStartup(514u16, &mut wsa_data as *mut WinSock::WSAData);
            if ret != 0 {
                println!("error 1: {}", ret);
                return;
            }

            ListenSocket = WinSock::socket(WinSock::AF_UNIX.into(), WinSock::SOCK_STREAM.into(), 0);
            if ListenSocket == WinSock::INVALID_SOCKET {
                println!("error 2: {:?}", WinSock::WSAGetLastError());
                return;
            }

            // TODO: Clean up
            let mut ServerSocket: WinSock::sockaddr_un = std::mem::zeroed();
            ServerSocket.sun_family = WinSock::AF_UNIX;
            let char_vec: Vec<char> = String::from(FIG_SOCKET_PATH).chars().collect();
            let mut byte_arr: [CHAR; 108] = std::mem::zeroed();
            for i in 0..char_vec.len() {
                byte_arr[i] = CHAR(char_vec[i] as u8);
            }
            ServerSocket.sun_path = byte_arr;

            ret = WinSock::bind(
                ListenSocket,
                unsafe {
                    std::mem::transmute::<*const WinSock::sockaddr_un, *const WinSock::SOCKADDR>(
                        &ServerSocket,
                    )
                },
                std::mem::size_of::<WinSock::sockaddr_un>()
                    .try_into()
                    .unwrap(),
            );

            if ret == WinSock::SOCKET_ERROR {
                println!("error 3: {:?}", WinSock::WSAGetLastError());
                return;
            }

            let w = Arc::new(Mutex::new(window));

            loop {
                ret = WinSock::listen(ListenSocket, WinSock::SOMAXCONN.try_into().unwrap());
                if ret == WinSock::SOCKET_ERROR {
                    println!("error 4: {:?}", WinSock::WSAGetLastError());
                    return;
                }
                println!("Accepting connections on {}", FIG_SOCKET_PATH);

                let mut addr: WinSock::SOCKADDR = std::mem::zeroed();
                let mut addrlen: i32 = std::mem::size_of::<WinSock::SOCKADDR>().try_into().unwrap();

                ClientSocket = WinSock::accept(
                    ListenSocket,
                    &mut addr as *mut WinSock::SOCKADDR,
                    &mut addrlen as *mut i32,
                );

                let p = w.clone();
                std::thread::spawn(move || {
                    if ClientSocket == WinSock::INVALID_SOCKET {
                        println!("error 5: {:?}", WinSock::WSAGetLastError());
                        return;
                    }
                    println!("Accepted a connection.");

                    loop {
                        let mut rec_buffer: [u8; 1024] = std::mem::zeroed();

                        let ResResult =
                            WinSock::recv(ClientSocket, PSTR(&mut rec_buffer as *mut u8), 1024, 0);
                        if ResResult == WinSock::SOCKET_ERROR {
                            println!("error 6: {:?}", WinSock::WSAGetLastError());
                            break;
                        }
                        if ResResult == 0 {
                            println!("EOF received");
                            break;
                        }

                        println!("{} bytes received", ResResult);

                        let mut cursor = Cursor::new(&rec_buffer[..]);
                        match FigMessage::parse(&mut cursor) {
                            Ok(message) => {
                                let msg = protobuf_decode::<LocalMessage>(message);
                                let msg_string = String::from(format!("{:?}", msg));
                                if let Type::Hook(hook) = msg.r#type.unwrap() {
                                    if let Hook::EditBuffer(e_buffer) = hook.hook.unwrap() {
                                        let session_id =
                                            e_buffer.context.unwrap().session_id.unwrap();
                                        p.lock().unwrap().emit("session_id", session_id);
                                    }
                                }
                                p.lock().unwrap().emit("figterm", msg_string);
                            }
                            Err(err) => {
                                println!("error {}", err);
                                break;
                            }
                        }
                    }
                    println!("Shutting down.");
                    ret = WinSock::shutdown(ClientSocket, 0);
                    if ret == WinSock::SOCKET_ERROR {
                        println!("error 7: {:?}", WinSock::WSAGetLastError());
                        return;
                    }
                });
            }
        }
    });
}

#[tauri::command]
fn insert_text(session_id: String, text: String) {
    println!("{} {}", session_id, text);

    unsafe {
        let FIGTERM_SOCKET_PATH = format!(r"C:\fig\figterm-{}.socket", session_id);
        let FIG_SOCKET_PATH = r"C:\fig\fig.socket";

        FileSystem::DeleteFileA(PCSTR(format!("{}\0", FIG_SOCKET_PATH).as_ptr()));

        let mut ListenSocket: WinSock::SOCKET = WinSock::INVALID_SOCKET;
        let mut ClientSocket: WinSock::SOCKET = WinSock::INVALID_SOCKET;
        let mut wsa_data: WinSock::WSAData = std::mem::zeroed();

        let mut ret: i32 = WinSock::WSAStartup(514u16, &mut wsa_data as *mut WinSock::WSAData);
        if ret != 0 {
            println!("error 1: {}", ret);
            return;
        }

        ClientSocket = WinSock::socket(WinSock::AF_UNIX.into(), WinSock::SOCK_STREAM.into(), 0);
        if ClientSocket == WinSock::INVALID_SOCKET {
            println!("error 2: {:?}", WinSock::WSAGetLastError());
            return;
        }

        let mut ClientAddr: WinSock::sockaddr_un = std::mem::zeroed();
        ClientAddr.sun_family = WinSock::AF_UNIX;
        let char_vec: Vec<char> = String::from(FIG_SOCKET_PATH).chars().collect();
        let mut byte_arr: [CHAR; 108] = std::mem::zeroed();
        for i in 0..char_vec.len() {
            byte_arr[i] = CHAR(char_vec[i] as u8);
        }
        ClientAddr.sun_path = byte_arr;

        ret = WinSock::bind(
            ClientSocket,
            unsafe {
                std::mem::transmute::<*const WinSock::sockaddr_un, *const WinSock::SOCKADDR>(
                    &ClientAddr,
                )
            },
            std::mem::size_of::<WinSock::sockaddr_un>()
                .try_into()
                .unwrap(),
        );

        if ret == WinSock::SOCKET_ERROR {
            println!("error 3: {:?}", WinSock::WSAGetLastError());
            return;
        }

        let mut ServerAddr: WinSock::sockaddr_un = std::mem::zeroed();
        ServerAddr.sun_family = WinSock::AF_UNIX;
        let char_vec: Vec<char> = String::from(FIGTERM_SOCKET_PATH).chars().collect();
        let mut byte_arr: [CHAR; 108] = std::mem::zeroed();
        for i in 0..char_vec.len() {
            byte_arr[i] = CHAR(char_vec[i] as u8);
        }
        ServerAddr.sun_path = byte_arr;

        ret = WinSock::connect(
            ClientSocket,
            unsafe {
                std::mem::transmute::<*const WinSock::sockaddr_un, *const WinSock::SOCKADDR>(
                    &ServerAddr,
                )
            },
            std::mem::size_of::<WinSock::sockaddr_un>()
                .try_into()
                .unwrap(),
        );

        if ret == WinSock::SOCKET_ERROR {
            println!("error 4: {:?}", WinSock::WSAGetLastError());
            return;
        }

        println!("Successfully connected");

        let cmd: InsertTextCommand = InsertTextCommand {
            insertion: Some(text),
            deletion: None,
            offset: None,
            immediate: None,
        };
        let fterm_msg: FigtermMessage = FigtermMessage {
            r#command: Some(figterm_message::Command::InsertTextCommand(cmd)),
        };

        let ResResult = WinSock::send(
            ClientSocket,
            PCSTR(fterm_msg.encode_fig_protobuf().unwrap().deref().as_ptr()),
            1024,
            WinSock::SEND_FLAGS(0),
        );
        if ResResult == WinSock::SOCKET_ERROR {
            println!("error 5: {:?}", WinSock::WSAGetLastError());
            return;
        }
        println!("{} bytes sent", ResResult);

        println!("Shutting down.");

        ret = WinSock::shutdown(ClientSocket, 0);
        if ret == WinSock::SOCKET_ERROR {
            println!("error 6: {:?}", WinSock::WSAGetLastError());
            return;
        }
    }
}

#[tauri::command]
fn window_stream(window: Window, window_info_state: State<'_, WindowInfoPayload>) {
    println!("window stream init");
    let window_info_state_clone: Arc<Mutex<WindowInfo>> = window_info_state.0.clone();
    std::thread::spawn(move || loop {
        unsafe {
            CoInitialize(std::ptr::null()).unwrap();
            let hwnd: HWND = GetForegroundWindow();

            let window_id: u32 = hwnd.0 as u32;
            let process_id: u32 = get_process_id(hwnd);

            let new_window_info = WindowInfo {
                window_id: window_id,
                process_id: process_id,
                caret_pos: WindowBound::from(match get_caret_pos(hwnd) {
                    Ok(res) => res,
                    Err(_) => RECT::default(),
                }),
                window_pos: WindowBound::from(match get_window_pos(hwnd) {
                    Some(res) => res,
                    None => RECT::default(),
                }),
            };

            if *window_info_state_clone.lock().unwrap() != new_window_info {
                *window_info_state_clone.lock().unwrap() = new_window_info.clone();
                let _res = window.emit("wininfo", new_window_info);
            }
            CoUninitialize();
        }
    });
}

unsafe fn get_process_id(hwnd: HWND) -> u32 {
    let mut pid: u32 = std::mem::zeroed();
    let _parent_pid = GetWindowThreadProcessId(hwnd, &mut pid as *mut u32);
    pid
}

unsafe fn get_window_pos(hwnd: HWND) -> Option<RECT> {
    let mut win_pos: RECT = std::mem::zeroed();
    if GetWindowRect(hwnd, &mut win_pos as *mut RECT) == BOOL(0) {
        return None;
    }
    Some(win_pos)
}

unsafe fn get_caret_pos(hwnd: HWND) -> Result<RECT, Error> {
    let automation: IUIAutomation = CoCreateInstance(&CUIAutomation8, None, CLSCTX_INPROC_SERVER)?;

    let elt: IUIAutomationElement = automation.ElementFromHandle(hwnd)?;

    let terminal_input_pattern_condition =
        automation.CreatePropertyCondition(UIA_NamePropertyId, get_variant_caret_name())?;
    let caret_elt: IUIAutomationElement =
        elt.FindFirst(TreeScope_Descendants, terminal_input_pattern_condition)?;

    caret_elt.CurrentBoundingRectangle()
}

// TODO: Cleanup
fn protobuf_decode<T>(message: FigMessage) -> T
where
    T: Message + Default,
{
    return T::decode(message.as_ref()).unwrap();
}

// TODO: Cleanup
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
