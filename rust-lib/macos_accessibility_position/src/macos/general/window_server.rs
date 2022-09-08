pub trait WindowServer {
    unsafe fn register_observer(&self) -> bool;
    // unsafe fn ax_callback(observer: AXObserverRef, element: AXUIElement, notification_name: CFString,
    // refcon: *mut c_void);
    unsafe fn deregister_observer();
}
