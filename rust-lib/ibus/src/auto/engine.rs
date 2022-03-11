// This file was generated by gir (https://github.com/gtk-rs/gir)
// from gir-files
// DO NOT EDIT

use glib::object::Cast;
use glib::object::IsA;
use glib::signal::connect_raw;
use glib::signal::SignalHandlerId;
use glib::translate::*;
use glib::ToValue;
use std::boxed::Box as Box_;
use std::fmt;
use std::mem;
use std::mem::transmute;

glib::wrapper! {
    #[doc(alias = "IBusEngine")]
    pub struct Engine(Object<ffi::IBusEngine, ffi::IBusEngineClass>);

    match fn {
        type_ => || ffi::ibus_engine_get_type(),
    }
}

impl Engine {
    pub const NONE: Option<&'static Engine> = None;

    #[doc(alias = "ibus_engine_new")]
    pub fn new(engine_name: &str, object_path: &str, connection: &gio::DBusConnection) -> Engine {
        unsafe {
            from_glib_none(ffi::ibus_engine_new(
                engine_name.to_glib_none().0,
                object_path.to_glib_none().0,
                connection.to_glib_none().0,
            ))
        }
    }

    #[doc(alias = "ibus_engine_new_with_type")]
    #[doc(alias = "new_with_type")]
    pub fn with_type(
        engine_type: glib::types::Type,
        engine_name: &str,
        object_path: &str,
        connection: &gio::DBusConnection,
    ) -> Engine {
        unsafe {
            from_glib_none(ffi::ibus_engine_new_with_type(
                engine_type.into_glib(),
                engine_name.to_glib_none().0,
                object_path.to_glib_none().0,
                connection.to_glib_none().0,
            ))
        }
    }

    // rustdoc-stripper-ignore-next
    /// Creates a new builder-pattern struct instance to construct [`Engine`] objects.
    ///
    /// This method returns an instance of [`EngineBuilder`](crate::builders::EngineBuilder) which can be used to create [`Engine`] objects.
    pub fn builder() -> EngineBuilder {
        EngineBuilder::default()
    }
}

impl Default for Engine {
    fn default() -> Self {
        glib::object::Object::new::<Self>(&[])
            .expect("Can't construct Engine object with default parameters")
    }
}

#[derive(Clone, Default)]
// rustdoc-stripper-ignore-next
/// A [builder-pattern] type to construct [`Engine`] objects.
///
/// [builder-pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
#[must_use = "The builder must be built to be used"]
pub struct EngineBuilder {
    engine_name: Option<String>,
}

impl EngineBuilder {
    // rustdoc-stripper-ignore-next
    /// Create a new [`EngineBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    // rustdoc-stripper-ignore-next
    /// Build the [`Engine`].
    #[must_use = "Building the object from the builder is usually expensive and is not expected to have side effects"]
    pub fn build(self) -> Engine {
        let mut properties: Vec<(&str, &dyn ToValue)> = vec![];
        if let Some(ref engine_name) = self.engine_name {
            properties.push(("engine-name", engine_name));
        }
        glib::Object::new::<Engine>(&properties).expect("Failed to create an instance of Engine")
    }

    pub fn engine_name(mut self, engine_name: &str) -> Self {
        self.engine_name = Some(engine_name.to_string());
        self
    }
}

pub trait EngineExt: 'static {
    //#[doc(alias = "ibus_engine_commit_text")]
    //fn commit_text(&self, text: /*Ignored*/&Text);

    #[doc(alias = "ibus_engine_delete_surrounding_text")]
    fn delete_surrounding_text(&self, offset: i32, nchars: u32);

    #[doc(alias = "ibus_engine_forward_key_event")]
    fn forward_key_event(&self, keyval: u32, keycode: u32, state: u32);

    #[doc(alias = "ibus_engine_get_content_type")]
    #[doc(alias = "get_content_type")]
    fn content_type(&self) -> (u32, u32);

    #[doc(alias = "ibus_engine_get_name")]
    #[doc(alias = "get_name")]
    fn name(&self) -> Option<glib::GString>;

    //#[doc(alias = "ibus_engine_get_surrounding_text")]
    //#[doc(alias = "get_surrounding_text")]
    //fn surrounding_text(&self, text: /*Ignored*/Text) -> (u32, u32);

    #[doc(alias = "ibus_engine_hide_auxiliary_text")]
    fn hide_auxiliary_text(&self);

    #[doc(alias = "ibus_engine_hide_lookup_table")]
    fn hide_lookup_table(&self);

    #[doc(alias = "ibus_engine_hide_preedit_text")]
    fn hide_preedit_text(&self);

    //#[doc(alias = "ibus_engine_register_properties")]
    //fn register_properties(&self, prop_list: /*Ignored*/&PropList);

    #[doc(alias = "ibus_engine_show_auxiliary_text")]
    fn show_auxiliary_text(&self);

    #[doc(alias = "ibus_engine_show_lookup_table")]
    fn show_lookup_table(&self);

    #[doc(alias = "ibus_engine_show_preedit_text")]
    fn show_preedit_text(&self);

    //#[doc(alias = "ibus_engine_update_auxiliary_text")]
    //fn update_auxiliary_text(&self, text: /*Ignored*/&Text, visible: bool);

    //#[doc(alias = "ibus_engine_update_lookup_table")]
    //fn update_lookup_table(&self, lookup_table: /*Ignored*/&LookupTable, visible: bool);

    //#[doc(alias = "ibus_engine_update_lookup_table_fast")]
    //fn update_lookup_table_fast(&self, lookup_table: /*Ignored*/&LookupTable, visible: bool);

    //#[doc(alias = "ibus_engine_update_preedit_text")]
    //fn update_preedit_text(&self, text: /*Ignored*/&Text, cursor_pos: u32, visible: bool);

    //#[doc(alias = "ibus_engine_update_preedit_text_with_mode")]
    //fn update_preedit_text_with_mode(&self, text: /*Ignored*/&Text, cursor_pos: u32, visible: bool, mode: /*Ignored*/PreeditFocusMode);

    //#[doc(alias = "ibus_engine_update_property")]
    //fn update_property(&self, prop: /*Ignored*/&Property);

    #[doc(alias = "engine-name")]
    fn engine_name(&self) -> Option<glib::GString>;

    #[doc(alias = "cancel-hand-writing")]
    fn connect_cancel_hand_writing<F: Fn(&Self, u32) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "candidate-clicked")]
    fn connect_candidate_clicked<F: Fn(&Self, u32, u32, u32) + 'static>(
        &self,
        f: F,
    ) -> SignalHandlerId;

    #[doc(alias = "cursor-down")]
    fn connect_cursor_down<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "cursor-up")]
    fn connect_cursor_up<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "disable")]
    fn connect_disable<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "enable")]
    fn connect_enable<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "focus-in")]
    fn connect_focus_in<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "focus-out")]
    fn connect_focus_out<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "page-down")]
    fn connect_page_down<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "page-up")]
    fn connect_page_up<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId;

    //#[doc(alias = "process-hand-writing-event")]
    //fn connect_process_hand_writing_event<Unsupported or ignored types>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "process-key-event")]
    fn connect_process_key_event<F: Fn(&Self, u32, u32, u32) -> bool + 'static>(
        &self,
        f: F,
    ) -> SignalHandlerId;

    #[doc(alias = "property-activate")]
    fn connect_property_activate<F: Fn(&Self, &str, u32) + 'static>(&self, f: F)
        -> SignalHandlerId;

    #[doc(alias = "property-hide")]
    fn connect_property_hide<F: Fn(&Self, &str) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "property-show")]
    fn connect_property_show<F: Fn(&Self, &str) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "reset")]
    fn connect_reset<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "set-capabilities")]
    fn connect_set_capabilities<F: Fn(&Self, u32) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "set-content-type")]
    fn connect_set_content_type<F: Fn(&Self, u32, u32) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "set-cursor-location")]
    fn connect_set_cursor_location<F: Fn(&Self, i32, i32, i32, i32) + 'static>(
        &self,
        f: F,
    ) -> SignalHandlerId;

    //#[doc(alias = "set-surrounding-text")]
    //fn connect_set_surrounding_text<Unsupported or ignored types>(&self, f: F) -> SignalHandlerId;
}

impl<O: IsA<Engine>> EngineExt for O {
    //fn commit_text(&self, text: /*Ignored*/&Text) {
    //    unsafe { TODO: call ffi:ibus_engine_commit_text() }
    //}

    fn delete_surrounding_text(&self, offset: i32, nchars: u32) {
        unsafe {
            ffi::ibus_engine_delete_surrounding_text(
                self.as_ref().to_glib_none().0,
                offset,
                nchars,
            );
        }
    }

    fn forward_key_event(&self, keyval: u32, keycode: u32, state: u32) {
        unsafe {
            ffi::ibus_engine_forward_key_event(
                self.as_ref().to_glib_none().0,
                keyval,
                keycode,
                state,
            );
        }
    }

    fn content_type(&self) -> (u32, u32) {
        unsafe {
            let mut purpose = mem::MaybeUninit::uninit();
            let mut hints = mem::MaybeUninit::uninit();
            ffi::ibus_engine_get_content_type(
                self.as_ref().to_glib_none().0,
                purpose.as_mut_ptr(),
                hints.as_mut_ptr(),
            );
            let purpose = purpose.assume_init();
            let hints = hints.assume_init();
            (purpose, hints)
        }
    }

    fn name(&self) -> Option<glib::GString> {
        unsafe { from_glib_none(ffi::ibus_engine_get_name(self.as_ref().to_glib_none().0)) }
    }

    //fn surrounding_text(&self, text: /*Ignored*/Text) -> (u32, u32) {
    //    unsafe { TODO: call ffi:ibus_engine_get_surrounding_text() }
    //}

    fn hide_auxiliary_text(&self) {
        unsafe {
            ffi::ibus_engine_hide_auxiliary_text(self.as_ref().to_glib_none().0);
        }
    }

    fn hide_lookup_table(&self) {
        unsafe {
            ffi::ibus_engine_hide_lookup_table(self.as_ref().to_glib_none().0);
        }
    }

    fn hide_preedit_text(&self) {
        unsafe {
            ffi::ibus_engine_hide_preedit_text(self.as_ref().to_glib_none().0);
        }
    }

    //fn register_properties(&self, prop_list: /*Ignored*/&PropList) {
    //    unsafe { TODO: call ffi:ibus_engine_register_properties() }
    //}

    fn show_auxiliary_text(&self) {
        unsafe {
            ffi::ibus_engine_show_auxiliary_text(self.as_ref().to_glib_none().0);
        }
    }

    fn show_lookup_table(&self) {
        unsafe {
            ffi::ibus_engine_show_lookup_table(self.as_ref().to_glib_none().0);
        }
    }

    fn show_preedit_text(&self) {
        unsafe {
            ffi::ibus_engine_show_preedit_text(self.as_ref().to_glib_none().0);
        }
    }

    //fn update_auxiliary_text(&self, text: /*Ignored*/&Text, visible: bool) {
    //    unsafe { TODO: call ffi:ibus_engine_update_auxiliary_text() }
    //}

    //fn update_lookup_table(&self, lookup_table: /*Ignored*/&LookupTable, visible: bool) {
    //    unsafe { TODO: call ffi:ibus_engine_update_lookup_table() }
    //}

    //fn update_lookup_table_fast(&self, lookup_table: /*Ignored*/&LookupTable, visible: bool) {
    //    unsafe { TODO: call ffi:ibus_engine_update_lookup_table_fast() }
    //}

    //fn update_preedit_text(&self, text: /*Ignored*/&Text, cursor_pos: u32, visible: bool) {
    //    unsafe { TODO: call ffi:ibus_engine_update_preedit_text() }
    //}

    //fn update_preedit_text_with_mode(&self, text: /*Ignored*/&Text, cursor_pos: u32, visible: bool, mode: /*Ignored*/PreeditFocusMode) {
    //    unsafe { TODO: call ffi:ibus_engine_update_preedit_text_with_mode() }
    //}

    //fn update_property(&self, prop: /*Ignored*/&Property) {
    //    unsafe { TODO: call ffi:ibus_engine_update_property() }
    //}

    fn engine_name(&self) -> Option<glib::GString> {
        glib::ObjectExt::property(self.as_ref(), "engine-name")
    }

    fn connect_cancel_hand_writing<F: Fn(&Self, u32) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn cancel_hand_writing_trampoline<
            P: IsA<Engine>,
            F: Fn(&P, u32) + 'static,
        >(
            this: *mut ffi::IBusEngine,
            n_strokes: libc::c_uint,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Engine::from_glib_borrow(this).unsafe_cast_ref(), n_strokes)
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"cancel-hand-writing\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    cancel_hand_writing_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_candidate_clicked<F: Fn(&Self, u32, u32, u32) + 'static>(
        &self,
        f: F,
    ) -> SignalHandlerId {
        unsafe extern "C" fn candidate_clicked_trampoline<
            P: IsA<Engine>,
            F: Fn(&P, u32, u32, u32) + 'static,
        >(
            this: *mut ffi::IBusEngine,
            index: libc::c_uint,
            button: libc::c_uint,
            state: libc::c_uint,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(
                Engine::from_glib_borrow(this).unsafe_cast_ref(),
                index,
                button,
                state,
            )
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"candidate-clicked\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    candidate_clicked_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_cursor_down<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn cursor_down_trampoline<P: IsA<Engine>, F: Fn(&P) + 'static>(
            this: *mut ffi::IBusEngine,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Engine::from_glib_borrow(this).unsafe_cast_ref())
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"cursor-down\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    cursor_down_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_cursor_up<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn cursor_up_trampoline<P: IsA<Engine>, F: Fn(&P) + 'static>(
            this: *mut ffi::IBusEngine,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Engine::from_glib_borrow(this).unsafe_cast_ref())
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"cursor-up\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    cursor_up_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_disable<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn disable_trampoline<P: IsA<Engine>, F: Fn(&P) + 'static>(
            this: *mut ffi::IBusEngine,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Engine::from_glib_borrow(this).unsafe_cast_ref())
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"disable\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    disable_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_enable<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn enable_trampoline<P: IsA<Engine>, F: Fn(&P) + 'static>(
            this: *mut ffi::IBusEngine,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Engine::from_glib_borrow(this).unsafe_cast_ref())
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"enable\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    enable_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_focus_in<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn focus_in_trampoline<P: IsA<Engine>, F: Fn(&P) + 'static>(
            this: *mut ffi::IBusEngine,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Engine::from_glib_borrow(this).unsafe_cast_ref())
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"focus-in\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    focus_in_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_focus_out<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn focus_out_trampoline<P: IsA<Engine>, F: Fn(&P) + 'static>(
            this: *mut ffi::IBusEngine,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Engine::from_glib_borrow(this).unsafe_cast_ref())
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"focus-out\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    focus_out_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_page_down<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn page_down_trampoline<P: IsA<Engine>, F: Fn(&P) + 'static>(
            this: *mut ffi::IBusEngine,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Engine::from_glib_borrow(this).unsafe_cast_ref())
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"page-down\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    page_down_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_page_up<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn page_up_trampoline<P: IsA<Engine>, F: Fn(&P) + 'static>(
            this: *mut ffi::IBusEngine,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Engine::from_glib_borrow(this).unsafe_cast_ref())
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"page-up\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    page_up_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    //fn connect_process_hand_writing_event<Unsupported or ignored types>(&self, f: F) -> SignalHandlerId {
    //    Unimplemented coordinates: *.Pointer
    //}

    fn connect_process_key_event<F: Fn(&Self, u32, u32, u32) -> bool + 'static>(
        &self,
        f: F,
    ) -> SignalHandlerId {
        unsafe extern "C" fn process_key_event_trampoline<
            P: IsA<Engine>,
            F: Fn(&P, u32, u32, u32) -> bool + 'static,
        >(
            this: *mut ffi::IBusEngine,
            keyval: libc::c_uint,
            keycode: libc::c_uint,
            state: libc::c_uint,
            f: glib::ffi::gpointer,
        ) -> glib::ffi::gboolean {
            let f: &F = &*(f as *const F);
            f(
                Engine::from_glib_borrow(this).unsafe_cast_ref(),
                keyval,
                keycode,
                state,
            )
            .into_glib()
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"process-key-event\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    process_key_event_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_property_activate<F: Fn(&Self, &str, u32) + 'static>(
        &self,
        f: F,
    ) -> SignalHandlerId {
        unsafe extern "C" fn property_activate_trampoline<
            P: IsA<Engine>,
            F: Fn(&P, &str, u32) + 'static,
        >(
            this: *mut ffi::IBusEngine,
            name: *mut libc::c_char,
            state: libc::c_uint,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(
                Engine::from_glib_borrow(this).unsafe_cast_ref(),
                &glib::GString::from_glib_borrow(name),
                state,
            )
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"property-activate\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    property_activate_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_property_hide<F: Fn(&Self, &str) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn property_hide_trampoline<P: IsA<Engine>, F: Fn(&P, &str) + 'static>(
            this: *mut ffi::IBusEngine,
            name: *mut libc::c_char,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(
                Engine::from_glib_borrow(this).unsafe_cast_ref(),
                &glib::GString::from_glib_borrow(name),
            )
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"property-hide\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    property_hide_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_property_show<F: Fn(&Self, &str) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn property_show_trampoline<P: IsA<Engine>, F: Fn(&P, &str) + 'static>(
            this: *mut ffi::IBusEngine,
            name: *mut libc::c_char,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(
                Engine::from_glib_borrow(this).unsafe_cast_ref(),
                &glib::GString::from_glib_borrow(name),
            )
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"property-show\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    property_show_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_reset<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn reset_trampoline<P: IsA<Engine>, F: Fn(&P) + 'static>(
            this: *mut ffi::IBusEngine,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Engine::from_glib_borrow(this).unsafe_cast_ref())
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"reset\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    reset_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_set_capabilities<F: Fn(&Self, u32) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn set_capabilities_trampoline<
            P: IsA<Engine>,
            F: Fn(&P, u32) + 'static,
        >(
            this: *mut ffi::IBusEngine,
            caps: libc::c_uint,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Engine::from_glib_borrow(this).unsafe_cast_ref(), caps)
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"set-capabilities\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    set_capabilities_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_set_content_type<F: Fn(&Self, u32, u32) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn set_content_type_trampoline<
            P: IsA<Engine>,
            F: Fn(&P, u32, u32) + 'static,
        >(
            this: *mut ffi::IBusEngine,
            purpose: libc::c_uint,
            hints: libc::c_uint,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(
                Engine::from_glib_borrow(this).unsafe_cast_ref(),
                purpose,
                hints,
            )
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"set-content-type\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    set_content_type_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_set_cursor_location<F: Fn(&Self, i32, i32, i32, i32) + 'static>(
        &self,
        f: F,
    ) -> SignalHandlerId {
        unsafe extern "C" fn set_cursor_location_trampoline<
            P: IsA<Engine>,
            F: Fn(&P, i32, i32, i32, i32) + 'static,
        >(
            this: *mut ffi::IBusEngine,
            x: libc::c_int,
            y: libc::c_int,
            w: libc::c_int,
            h: libc::c_int,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Engine::from_glib_borrow(this).unsafe_cast_ref(), x, y, w, h)
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"set-cursor-location\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    set_cursor_location_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    //fn connect_set_surrounding_text<Unsupported or ignored types>(&self, f: F) -> SignalHandlerId {
    //    Ignored text: GObject.Object
    //}
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Engine")
    }
}
