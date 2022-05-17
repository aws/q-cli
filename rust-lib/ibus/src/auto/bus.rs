// This file was generated by gir (https://github.com/gtk-rs/gir)
// from gir-files
// DO NOT EDIT

use std::boxed::Box as Box_;
use std::fmt;
use std::mem::transmute;

use glib::object::{
    Cast,
    IsA,
};
use glib::signal::{
    connect_raw,
    SignalHandlerId,
};
use glib::translate::*;
use glib::{
    StaticType,
    ToValue,
};

use crate::Component;

glib::wrapper! {
    #[doc(alias = "IBusBus")]
    pub struct Bus(Object<ffi::IBusBus, ffi::IBusBusClass>);

    match fn {
        type_ => || ffi::ibus_bus_get_type(),
    }
}

impl Bus {
    pub const NONE: Option<&'static Bus> = None;

    #[doc(alias = "ibus_bus_new")]
    pub fn new() -> Bus {
        unsafe { from_glib_none(ffi::ibus_bus_new()) }
    }

    #[doc(alias = "ibus_bus_new_async")]
    pub fn new_async() -> Bus {
        unsafe { from_glib_none(ffi::ibus_bus_new_async()) }
    }

    #[doc(alias = "ibus_bus_new_async_client")]
    pub fn new_async_client() -> Bus {
        unsafe { from_glib_none(ffi::ibus_bus_new_async_client()) }
    }

    // rustdoc-stripper-ignore-next
    /// Creates a new builder-pattern struct instance to construct [`Bus`] objects.
    ///
    /// This method returns an instance of [`BusBuilder`](crate::builders::BusBuilder) which can be
    /// used to create [`Bus`] objects.
    pub fn builder() -> BusBuilder {
        BusBuilder::default()
    }
}

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Default)]
// rustdoc-stripper-ignore-next
/// A [builder-pattern] type to construct [`Bus`] objects.
///
/// [builder-pattern]: https://doc.rust-lang.org/1.0.0/style/ownership/builders.html
#[must_use = "The builder must be built to be used"]
pub struct BusBuilder {
    client_only: Option<bool>,
    connect_async: Option<bool>,
}

impl BusBuilder {
    // rustdoc-stripper-ignore-next
    /// Create a new [`BusBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    // rustdoc-stripper-ignore-next
    /// Build the [`Bus`].
    #[must_use = "Building the object from the builder is usually expensive and is not expected to have side effects"]
    pub fn build(self) -> Bus {
        let mut properties: Vec<(&str, &dyn ToValue)> = vec![];
        if let Some(ref client_only) = self.client_only {
            properties.push(("client-only", client_only));
        }
        if let Some(ref connect_async) = self.connect_async {
            properties.push(("connect-async", connect_async));
        }
        glib::Object::new::<Bus>(&properties).expect("Failed to create an instance of Bus")
    }

    pub fn client_only(mut self, client_only: bool) -> Self {
        self.client_only = Some(client_only);
        self
    }

    pub fn connect_async(mut self, connect_async: bool) -> Self {
        self.connect_async = Some(connect_async);
        self
    }
}

pub trait BusExt: 'static {
    #[doc(alias = "ibus_bus_add_match")]
    fn add_match(&self, rule: &str) -> bool;

    //#[doc(alias = "ibus_bus_add_match_async")]
    // fn add_match_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, rule: &str, timeout_msec:
    // i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    //#[doc(alias = "ibus_bus_create_input_context")]
    // fn create_input_context(&self, client_name: &str) -> /*Ignored*/Option<InputContext>;

    //#[doc(alias = "ibus_bus_create_input_context_async")]
    // fn create_input_context_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, client_name:
    // &str, timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[doc(alias = "ibus_bus_current_input_context")]
    fn current_input_context(&self) -> Option<glib::GString>;

    //#[doc(alias = "ibus_bus_current_input_context_async")]
    // fn current_input_context_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, timeout_msec:
    // i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[doc(alias = "ibus_bus_exit")]
    fn exit(&self, restart: bool) -> bool;

    //#[doc(alias = "ibus_bus_exit_async")]
    // fn exit_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, restart: bool, timeout_msec:
    // i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    //#[doc(alias = "ibus_bus_get_config")]
    //#[doc(alias = "get_config")]
    // fn config(&self) -> /*Ignored*/Option<Config>;

    #[doc(alias = "ibus_bus_get_connection")]
    #[doc(alias = "get_connection")]
    fn connection(&self) -> Option<gio::DBusConnection>;

    //#[doc(alias = "ibus_bus_get_engines_by_names")]
    //#[doc(alias = "get_engines_by_names")]
    // fn engines_by_names(&self, names: &[&str]) -> /*Ignored*/Vec<EngineDesc>;

    //#[doc(alias = "ibus_bus_get_global_engine")]
    //#[doc(alias = "get_global_engine")]
    // fn global_engine(&self) -> /*Ignored*/Option<EngineDesc>;

    //#[doc(alias = "ibus_bus_get_global_engine_async")]
    //#[doc(alias = "get_global_engine_async")]
    // fn global_engine_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, timeout_msec: i32,
    // cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    //#[doc(alias = "ibus_bus_get_ibus_property")]
    //#[doc(alias = "get_ibus_property")]
    // fn ibus_property(&self, property_name: &str) -> /*Ignored*/Option<glib::Variant>;

    //#[doc(alias = "ibus_bus_get_ibus_property_async")]
    //#[doc(alias = "get_ibus_property_async")]
    // fn ibus_property_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, property_name: &str,
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[doc(alias = "ibus_bus_get_name_owner")]
    #[doc(alias = "get_name_owner")]
    fn name_owner(&self, name: &str) -> Option<glib::GString>;

    //#[doc(alias = "ibus_bus_get_name_owner_async")]
    //#[doc(alias = "get_name_owner_async")]
    // fn name_owner_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, name: &str,
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[doc(alias = "ibus_bus_get_service_name")]
    #[doc(alias = "get_service_name")]
    fn service_name(&self) -> Option<glib::GString>;

    #[cfg_attr(feature = "v1_5_3", deprecated = "Since 1.5.3")]
    #[doc(alias = "ibus_bus_get_use_global_engine")]
    #[doc(alias = "get_use_global_engine")]
    fn uses_global_engine(&self) -> bool;

    //#[cfg_attr(feature = "v1_5_3", deprecated = "Since 1.5.3")]
    //#[doc(alias = "ibus_bus_get_use_global_engine_async")]
    //#[doc(alias = "get_use_global_engine_async")]
    // fn use_global_engine_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, timeout_msec:
    // i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[cfg_attr(feature = "v1_5_3", deprecated = "Since 1.5.3")]
    #[doc(alias = "ibus_bus_get_use_sys_layout")]
    #[doc(alias = "get_use_sys_layout")]
    fn uses_sys_layout(&self) -> bool;

    //#[cfg_attr(feature = "v1_5_3", deprecated = "Since 1.5.3")]
    //#[doc(alias = "ibus_bus_get_use_sys_layout_async")]
    //#[doc(alias = "get_use_sys_layout_async")]
    // fn use_sys_layout_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, timeout_msec: i32,
    // cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[doc(alias = "ibus_bus_hello")]
    fn hello(&self) -> Option<glib::GString>;

    #[doc(alias = "ibus_bus_is_connected")]
    fn is_connected(&self) -> bool;

    #[cfg_attr(feature = "v1_5_3", deprecated = "Since 1.5.3")]
    #[doc(alias = "ibus_bus_is_global_engine_enabled")]
    fn is_global_engine_enabled(&self) -> bool;

    //#[cfg_attr(feature = "v1_5_3", deprecated = "Since 1.5.3")]
    //#[doc(alias = "ibus_bus_is_global_engine_enabled_async")]
    // fn is_global_engine_enabled_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self,
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    //#[cfg_attr(feature = "v1_5_3", deprecated = "Since 1.5.3")]
    //#[doc(alias = "ibus_bus_list_active_engines")]
    // fn list_active_engines(&self) -> /*Ignored*/Vec<EngineDesc>;

    //#[cfg_attr(feature = "v1_5_3", deprecated = "Since 1.5.3")]
    //#[doc(alias = "ibus_bus_list_active_engines_async")]
    // fn list_active_engines_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, timeout_msec:
    // i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    //#[doc(alias = "ibus_bus_list_engines")]
    // fn list_engines(&self) -> /*Ignored*/Vec<EngineDesc>;

    //#[doc(alias = "ibus_bus_list_engines_async")]
    // fn list_engines_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, timeout_msec: i32,
    // cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[doc(alias = "ibus_bus_list_names")]
    fn list_names(&self) -> Vec<glib::GString>;

    #[doc(alias = "ibus_bus_list_queued_owners")]
    fn list_queued_owners(&self, name: &str) -> Vec<glib::GString>;

    #[doc(alias = "ibus_bus_name_has_owner")]
    fn name_has_owner(&self, name: &str) -> bool;

    //#[doc(alias = "ibus_bus_name_has_owner_async")]
    // fn name_has_owner_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, name: &str,
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[doc(alias = "ibus_bus_preload_engines")]
    fn preload_engines(&self, names: &[&str]) -> bool;

    //#[doc(alias = "ibus_bus_preload_engines_async")]
    // fn preload_engines_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, names: &[&str],
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[doc(alias = "ibus_bus_register_component")]
    fn register_component(&self, component: &impl IsA<Component>) -> bool;

    //#[doc(alias = "ibus_bus_register_component_async")]
    // fn register_component_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, component: &impl
    // IsA<Component>, timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback:
    // P);

    #[doc(alias = "ibus_bus_release_name")]
    fn release_name(&self, name: &str) -> u32;

    //#[doc(alias = "ibus_bus_release_name_async")]
    // fn release_name_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, name: &str,
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[doc(alias = "ibus_bus_remove_match")]
    fn remove_match(&self, rule: &str) -> bool;

    //#[doc(alias = "ibus_bus_remove_match_async")]
    // fn remove_match_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, rule: &str,
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[doc(alias = "ibus_bus_request_name")]
    fn request_name(&self, name: &str, flags: u32) -> u32;

    //#[doc(alias = "ibus_bus_request_name_async")]
    // fn request_name_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, name: &str, flags:
    // u32, timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[doc(alias = "ibus_bus_set_global_engine")]
    fn set_global_engine(&self, global_engine: &str) -> bool;

    //#[doc(alias = "ibus_bus_set_global_engine_async")]
    // fn set_global_engine_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, global_engine:
    // &str, timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P);

    //#[doc(alias = "ibus_bus_set_ibus_property")]
    // fn set_ibus_property(&self, property_name: &str, value: /*Ignored*/&glib::Variant);

    //#[doc(alias = "ibus_bus_set_ibus_property_async")]
    // fn set_ibus_property_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, property_name:
    // &str, value: /*Ignored*/&glib::Variant, timeout_msec: i32, cancellable:
    // /*Ignored*/Option<&gio::Cancellable>, callback: P);

    #[doc(alias = "ibus_bus_set_watch_dbus_signal")]
    fn set_watch_dbus_signal(&self, watch: bool);

    #[doc(alias = "ibus_bus_set_watch_ibus_signal")]
    fn set_watch_ibus_signal(&self, watch: bool);

    #[doc(alias = "client-only")]
    fn is_client_only(&self) -> bool;

    #[doc(alias = "connect-async")]
    fn is_connect_async(&self) -> bool;

    #[doc(alias = "connected")]
    fn connect_connected<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "disconnected")]
    fn connect_disconnected<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "global-engine-changed")]
    fn connect_global_engine_changed<F: Fn(&Self, &str) + 'static>(&self, f: F) -> SignalHandlerId;

    #[doc(alias = "name-owner-changed")]
    fn connect_name_owner_changed<F: Fn(&Self, &str, &str, &str) + 'static>(&self, f: F) -> SignalHandlerId;
}

impl<O: IsA<Bus>> BusExt for O {
    fn add_match(&self, rule: &str) -> bool {
        unsafe {
            from_glib(ffi::ibus_bus_add_match(
                self.as_ref().to_glib_none().0,
                rule.to_glib_none().0,
            ))
        }
    }

    // fn add_match_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, rule: &str, timeout_msec:
    // i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {    unsafe { TODO: call
    // ffi:ibus_bus_add_match_async() }
    //}

    // fn create_input_context(&self, client_name: &str) -> /*Ignored*/Option<InputContext> {
    //    unsafe { TODO: call ffi:ibus_bus_create_input_context() }
    //}

    // fn create_input_context_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, client_name:
    // &str, timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {
    //    unsafe { TODO: call ffi:ibus_bus_create_input_context_async() }
    //}

    fn current_input_context(&self) -> Option<glib::GString> {
        unsafe { from_glib_full(ffi::ibus_bus_current_input_context(self.as_ref().to_glib_none().0)) }
    }

    // fn current_input_context_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, timeout_msec:
    // i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {    unsafe { TODO: call
    // ffi:ibus_bus_current_input_context_async() }
    //}

    fn exit(&self, restart: bool) -> bool {
        unsafe { from_glib(ffi::ibus_bus_exit(self.as_ref().to_glib_none().0, restart.into_glib())) }
    }

    // fn exit_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, restart: bool, timeout_msec:
    // i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {    unsafe { TODO: call
    // ffi:ibus_bus_exit_async() }
    //}

    // fn config(&self) -> /*Ignored*/Option<Config> {
    //    unsafe { TODO: call ffi:ibus_bus_get_config() }
    //}

    fn connection(&self) -> Option<gio::DBusConnection> {
        unsafe { from_glib_none(ffi::ibus_bus_get_connection(self.as_ref().to_glib_none().0)) }
    }

    // fn engines_by_names(&self, names: &[&str]) -> /*Ignored*/Vec<EngineDesc> {
    //    unsafe { TODO: call ffi:ibus_bus_get_engines_by_names() }
    //}

    // fn global_engine(&self) -> /*Ignored*/Option<EngineDesc> {
    //    unsafe { TODO: call ffi:ibus_bus_get_global_engine() }
    //}

    // fn global_engine_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, timeout_msec: i32,
    // cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {    unsafe { TODO: call
    // ffi:ibus_bus_get_global_engine_async() }
    //}

    // fn ibus_property(&self, property_name: &str) -> /*Ignored*/Option<glib::Variant> {
    //    unsafe { TODO: call ffi:ibus_bus_get_ibus_property() }
    //}

    // fn ibus_property_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, property_name: &str,
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {
    //    unsafe { TODO: call ffi:ibus_bus_get_ibus_property_async() }
    //}

    fn name_owner(&self, name: &str) -> Option<glib::GString> {
        unsafe {
            from_glib_full(ffi::ibus_bus_get_name_owner(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
            ))
        }
    }

    // fn name_owner_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, name: &str,
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {
    //    unsafe { TODO: call ffi:ibus_bus_get_name_owner_async() }
    //}

    fn service_name(&self) -> Option<glib::GString> {
        unsafe { from_glib_none(ffi::ibus_bus_get_service_name(self.as_ref().to_glib_none().0)) }
    }

    fn uses_global_engine(&self) -> bool {
        unsafe { from_glib(ffi::ibus_bus_get_use_global_engine(self.as_ref().to_glib_none().0)) }
    }

    // fn use_global_engine_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, timeout_msec:
    // i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {    unsafe { TODO: call
    // ffi:ibus_bus_get_use_global_engine_async() }
    //}

    fn uses_sys_layout(&self) -> bool {
        unsafe { from_glib(ffi::ibus_bus_get_use_sys_layout(self.as_ref().to_glib_none().0)) }
    }

    // fn use_sys_layout_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, timeout_msec: i32,
    // cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {    unsafe { TODO: call
    // ffi:ibus_bus_get_use_sys_layout_async() }
    //}

    fn hello(&self) -> Option<glib::GString> {
        unsafe { from_glib_none(ffi::ibus_bus_hello(self.as_ref().to_glib_none().0)) }
    }

    fn is_connected(&self) -> bool {
        unsafe { from_glib(ffi::ibus_bus_is_connected(self.as_ref().to_glib_none().0)) }
    }

    fn is_global_engine_enabled(&self) -> bool {
        unsafe { from_glib(ffi::ibus_bus_is_global_engine_enabled(self.as_ref().to_glib_none().0)) }
    }

    // fn is_global_engine_enabled_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self,
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {
    //    unsafe { TODO: call ffi:ibus_bus_is_global_engine_enabled_async() }
    //}

    // fn list_active_engines(&self) -> /*Ignored*/Vec<EngineDesc> {
    //    unsafe { TODO: call ffi:ibus_bus_list_active_engines() }
    //}

    // fn list_active_engines_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, timeout_msec:
    // i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {    unsafe { TODO: call
    // ffi:ibus_bus_list_active_engines_async() }
    //}

    // fn list_engines(&self) -> /*Ignored*/Vec<EngineDesc> {
    //    unsafe { TODO: call ffi:ibus_bus_list_engines() }
    //}

    // fn list_engines_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, timeout_msec: i32,
    // cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {    unsafe { TODO: call
    // ffi:ibus_bus_list_engines_async() }
    //}

    fn list_names(&self) -> Vec<glib::GString> {
        unsafe { FromGlibPtrContainer::from_glib_full(ffi::ibus_bus_list_names(self.as_ref().to_glib_none().0)) }
    }

    fn list_queued_owners(&self, name: &str) -> Vec<glib::GString> {
        unsafe {
            FromGlibPtrContainer::from_glib_full(ffi::ibus_bus_list_queued_owners(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
            ))
        }
    }

    fn name_has_owner(&self, name: &str) -> bool {
        unsafe {
            from_glib(ffi::ibus_bus_name_has_owner(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
            ))
        }
    }

    // fn name_has_owner_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, name: &str,
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {
    //    unsafe { TODO: call ffi:ibus_bus_name_has_owner_async() }
    //}

    fn preload_engines(&self, names: &[&str]) -> bool {
        unsafe {
            from_glib(ffi::ibus_bus_preload_engines(
                self.as_ref().to_glib_none().0,
                names.to_glib_none().0,
            ))
        }
    }

    // fn preload_engines_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, names: &[&str],
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {
    //    unsafe { TODO: call ffi:ibus_bus_preload_engines_async() }
    //}

    fn register_component(&self, component: &impl IsA<Component>) -> bool {
        unsafe {
            from_glib(ffi::ibus_bus_register_component(
                self.as_ref().to_glib_none().0,
                component.as_ref().to_glib_none().0,
            ))
        }
    }

    // fn register_component_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, component: &impl
    // IsA<Component>, timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback:
    // P) {    unsafe { TODO: call ffi:ibus_bus_register_component_async() }
    //}

    fn release_name(&self, name: &str) -> u32 {
        unsafe { ffi::ibus_bus_release_name(self.as_ref().to_glib_none().0, name.to_glib_none().0) }
    }

    // fn release_name_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, name: &str,
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {
    //    unsafe { TODO: call ffi:ibus_bus_release_name_async() }
    //}

    fn remove_match(&self, rule: &str) -> bool {
        unsafe {
            from_glib(ffi::ibus_bus_remove_match(
                self.as_ref().to_glib_none().0,
                rule.to_glib_none().0,
            ))
        }
    }

    // fn remove_match_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, rule: &str,
    // timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {
    //    unsafe { TODO: call ffi:ibus_bus_remove_match_async() }
    //}

    fn request_name(&self, name: &str, flags: u32) -> u32 {
        unsafe { ffi::ibus_bus_request_name(self.as_ref().to_glib_none().0, name.to_glib_none().0, flags) }
    }

    // fn request_name_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, name: &str, flags:
    // u32, timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {
    //    unsafe { TODO: call ffi:ibus_bus_request_name_async() }
    //}

    fn set_global_engine(&self, global_engine: &str) -> bool {
        unsafe {
            from_glib(ffi::ibus_bus_set_global_engine(
                self.as_ref().to_glib_none().0,
                global_engine.to_glib_none().0,
            ))
        }
    }

    // fn set_global_engine_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, global_engine:
    // &str, timeout_msec: i32, cancellable: /*Ignored*/Option<&gio::Cancellable>, callback: P) {
    //    unsafe { TODO: call ffi:ibus_bus_set_global_engine_async() }
    //}

    // fn set_ibus_property(&self, property_name: &str, value: /*Ignored*/&glib::Variant) {
    //    unsafe { TODO: call ffi:ibus_bus_set_ibus_property() }
    //}

    // fn set_ibus_property_async<P: FnOnce(Result<(), glib::Error>) + 'static>(&self, property_name:
    // &str, value: /*Ignored*/&glib::Variant, timeout_msec: i32, cancellable:
    // /*Ignored*/Option<&gio::Cancellable>, callback: P) {    unsafe { TODO: call
    // ffi:ibus_bus_set_ibus_property_async() }
    //}

    fn set_watch_dbus_signal(&self, watch: bool) {
        unsafe {
            ffi::ibus_bus_set_watch_dbus_signal(self.as_ref().to_glib_none().0, watch.into_glib());
        }
    }

    fn set_watch_ibus_signal(&self, watch: bool) {
        unsafe {
            ffi::ibus_bus_set_watch_ibus_signal(self.as_ref().to_glib_none().0, watch.into_glib());
        }
    }

    fn is_client_only(&self) -> bool {
        glib::ObjectExt::property(self.as_ref(), "client-only")
    }

    fn is_connect_async(&self) -> bool {
        glib::ObjectExt::property(self.as_ref(), "connect-async")
    }

    fn connect_connected<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn connected_trampoline<P: IsA<Bus>, F: Fn(&P) + 'static>(
            this: *mut ffi::IBusBus,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Bus::from_glib_borrow(this).unsafe_cast_ref())
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"connected\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    connected_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_disconnected<F: Fn(&Self) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn disconnected_trampoline<P: IsA<Bus>, F: Fn(&P) + 'static>(
            this: *mut ffi::IBusBus,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(Bus::from_glib_borrow(this).unsafe_cast_ref())
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"disconnected\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    disconnected_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_global_engine_changed<F: Fn(&Self, &str) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn global_engine_changed_trampoline<P: IsA<Bus>, F: Fn(&P, &str) + 'static>(
            this: *mut ffi::IBusBus,
            name: *mut libc::c_char,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(
                Bus::from_glib_borrow(this).unsafe_cast_ref(),
                &glib::GString::from_glib_borrow(name),
            )
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"global-engine-changed\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    global_engine_changed_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }

    fn connect_name_owner_changed<F: Fn(&Self, &str, &str, &str) + 'static>(&self, f: F) -> SignalHandlerId {
        unsafe extern "C" fn name_owner_changed_trampoline<P: IsA<Bus>, F: Fn(&P, &str, &str, &str) + 'static>(
            this: *mut ffi::IBusBus,
            name: *mut libc::c_char,
            old_owner: *mut libc::c_char,
            new_owner: *mut libc::c_char,
            f: glib::ffi::gpointer,
        ) {
            let f: &F = &*(f as *const F);
            f(
                Bus::from_glib_borrow(this).unsafe_cast_ref(),
                &glib::GString::from_glib_borrow(name),
                &glib::GString::from_glib_borrow(old_owner),
                &glib::GString::from_glib_borrow(new_owner),
            )
        }
        unsafe {
            let f: Box_<F> = Box_::new(f);
            connect_raw(
                self.as_ptr() as *mut _,
                b"name-owner-changed\0".as_ptr() as *const _,
                Some(transmute::<_, unsafe extern "C" fn()>(
                    name_owner_changed_trampoline::<Self, F> as *const (),
                )),
                Box_::into_raw(f),
            )
        }
    }
}

impl fmt::Display for Bus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("Bus")
    }
}
