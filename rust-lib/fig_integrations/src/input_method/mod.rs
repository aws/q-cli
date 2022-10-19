use std::borrow::Cow;
use std::os::unix::fs::symlink;
use std::path::{
    Path,
    PathBuf,
};
use std::process::Command;
use std::{
    fs,
    ptr,
};

use core_foundation::array::{
    CFArray,
    CFArrayRef,
};
use core_foundation::base::{
    Boolean,
    CFGetTypeID,
    CFTypeID,
    CFTypeRef,
    OSStatus,
    TCFType,
    TCFTypeRef,
};
use core_foundation::boolean::CFBoolean;
use core_foundation::bundle::{
    CFBundle,
    CFBundleRef,
};
use core_foundation::dictionary::{
    CFDictionary,
    CFDictionaryRef,
};
use core_foundation::string::{
    CFString,
    CFStringRef,
};
use core_foundation::url::{
    CFURLRef,
    CFURL,
};
use core_foundation::{
    declare_TCFType,
    impl_TCFType,
};
use fig_util::consts::FIG_CLI_BINARY_NAME;
use fig_util::directories::home_dir;
use objc::runtime::Object;
use objc::{
    class,
    msg_send,
    sel,
    sel_impl,
};
use serde::{
    Deserialize,
    Serialize,
};
use tracing::{
    debug,
    info,
    trace,
};

use crate::error::Result;
use crate::Integration;

pub enum __TISInputSource {}
pub type TISInputSourceRef = *const __TISInputSource;

declare_TCFType! {
    TISInputSource, TISInputSourceRef
}
impl_TCFType!(TISInputSource, TISInputSourceRef, TISInputSourceGetTypeID);

// https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.6.sdk/System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/Headers/TextInputSources.h
#[link(name = "Carbon", kind = "framework")]
extern "C" {
    pub static kTISPropertyBundleID: CFStringRef;
    pub static kTISPropertyInputSourceCategory: CFStringRef;
    pub static kTISPropertyInputSourceType: CFStringRef;
    pub static kTISPropertyInputSourceID: CFStringRef;
    pub static kTISPropertyInputSourceIsEnabled: CFStringRef;
    pub static kTISPropertyInputSourceIsSelected: CFStringRef;
    pub static kTISPropertyInputSourceIsEnableCapable: CFStringRef;
    pub static kTISPropertyInputSourceIsSelectCapable: CFStringRef;
    pub static kTISPropertyLocalizedName: CFStringRef;
    pub static kTISPropertyInputModeID: CFStringRef;

    // Can not be used as properties to filter TISCreateInputSourceList
    pub static kTISCategoryKeyboardInputSource: CFStringRef;

    pub static kTISNotifySelectedKeyboardInputSourceChanged: CFStringRef;

    pub static kTISNotifyEnabledKeyboardInputSourcesChanged: CFStringRef;

    pub fn TISInputSourceGetTypeID() -> CFTypeID;

    pub fn TISCreateInputSourceList(properties: CFDictionaryRef, include_all_installed: bool) -> CFArrayRef;

    pub fn TISGetInputSourceProperty(input_source: TISInputSourceRef, property_key: CFStringRef) -> CFTypeRef;

    pub fn TISSelectInputSource(input_source: TISInputSourceRef) -> OSStatus;

    pub fn TISDeselectInputSource(input_source: TISInputSourceRef) -> OSStatus;

    pub fn TISEnableInputSource(input_source: TISInputSourceRef) -> OSStatus;

    pub fn TISDisableInputSource(input_source: TISInputSourceRef) -> OSStatus;

    pub fn TISRegisterInputSource(location: CFURLRef) -> OSStatus;
}

pub struct InputMethod {
    pub bundle_path: PathBuf,
}

use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
pub enum InputMethodError {
    #[error("Could not list input sources")]
    CouldNotListInputSources,
    #[error("No input sources for bundle identifier '{0}'")]
    NoInputSourcesForBundleIdentifier(Cow<'static, str>),
    #[error("Found {0} input sources for bundle identifier")]
    MultipleInputSourcesForBundleIdentifier(usize),
    #[error("Invalid input method bundle destination")]
    InvalidDestination,
    #[error("Invalid path to bundle. Perhaps use an absolute path instead?")]
    InvalidBundlePath,
    #[error("Invalid input method bundle: {0}")]
    InvalidBundle(Cow<'static, str>),
    #[error("OSStatus error code: {0}")]
    OSStatusError(OSStatus),
    #[error("Input source is not enabled")]
    NotEnabled,
    #[error("Input source is not selected")]
    NotSelected,
    #[error("Could not locate Fig CLI")]
    HelperExecutableNotFound,
}

#[macro_export]
macro_rules! tis_action {
    ($action:ident, $function:ident) => {
        pub fn $action(&self) -> Result<(), InputMethodError> {
            debug!("{} input source.", stringify!($action));
            unsafe {
                match $function(self.as_concrete_TypeRef()) {
                    0 => Ok(()),
                    i => Err(InputMethodError::OSStatusError(i).into()),
                }
            }
        }
    };
}

#[macro_export]
macro_rules! tis_property {
    ($name:ident, $tis_property_key:expr, $cf_type:ty, $rust_type:ty, $convert:ident) => {
        #[allow(dead_code)]
        pub fn $name(&self) -> Option<$rust_type> {
            trace!("Get '{}' from input source", stringify!($name));
            self.get_property::<$cf_type>(unsafe { $tis_property_key })
                .map(|s| s.$convert())
        }
    };
}

#[macro_export]
macro_rules! tis_bool_property {
    ($name:ident, $tis_property_key:expr) => {
        tis_property!($name, $tis_property_key, CFBoolean, bool, into);
    };
}

#[macro_export]
macro_rules! tis_string_property {
    ($name:ident, $tis_property_key:expr) => {
        tis_property!($name, $tis_property_key, CFString, String, to_string);
    };
}

impl TISInputSource {
    tis_string_property!(bundle_id, kTISPropertyBundleID);

    tis_string_property!(input_source_id, kTISPropertyInputSourceID);

    tis_string_property!(category, kTISPropertyInputSourceCategory);

    tis_string_property!(localized_name, kTISPropertyLocalizedName);

    tis_string_property!(input_mode_id, kTISPropertyInputModeID);

    tis_string_property!(category_keyboard, kTISCategoryKeyboardInputSource);

    tis_bool_property!(is_enabled, kTISPropertyInputSourceIsEnabled);

    tis_bool_property!(is_enable_capable, kTISPropertyInputSourceIsEnableCapable);

    tis_bool_property!(is_selected, kTISPropertyInputSourceIsSelected);

    tis_bool_property!(is_select_capable, kTISPropertyInputSourceIsSelectCapable);

    tis_action!(enable, TISEnableInputSource);

    tis_action!(disable, TISDisableInputSource);

    tis_action!(select, TISSelectInputSource);

    tis_action!(deselect, TISDeselectInputSource);

    // TODO: change to use FromVoid
    fn get_property<T: TCFType>(&self, key: CFStringRef) -> Option<T> {
        unsafe {
            let value: CFTypeRef = TISGetInputSourceProperty(self.as_concrete_TypeRef(), key);

            if value.is_null() {
                None
            } else if T::type_id() == CFGetTypeID(value) {
                // This has to be under get rule
                // https://github.com/phracker/MacOSX-SDKs/blob/master/MacOSX10.6.sdk/System/Library/Frameworks/Carbon.framework/Versions/A/Frameworks/HIToolbox.framework/Versions/A/Headers/TextInputSources.h#L695
                let value = <T::Ref as TCFTypeRef>::from_void_ptr(value);
                Some(T::wrap_under_get_rule(value))
            } else {
                None
            }
        }
    }
}

impl std::fmt::Debug for TISInputSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TISInputSource")
            .field("bundle_id", &self.bundle_id())
            .field("input_source_id", &self.input_source_id())
            .field("input_source_category", &self.category())
            .field("input_source_is_enabled", &self.is_enabled())
            .field("input_source_is_selected", &self.is_selected())
            .field("localized_name", &self.localized_name())
            .field("input_mode_id", &self.input_mode_id())
            .field("category_keyboard", &self.category_keyboard())
            .finish()
    }
}

impl std::default::Default for InputMethod {
    fn default() -> Self {
        let fig_app_path = fig_util::fig_bundle().unwrap_or_else(|| "/Applications/Fig.app".to_owned().into());

        let bundle_path = fig_app_path.join("Contents").join("Helpers").join("FigInputMethod.app");
        Self { bundle_path }
    }
}

impl InputMethod {
    pub fn input_method_directory() -> PathBuf {
        home_dir().ok().unwrap().join("Library").join("Input Methods")
    }

    pub fn list_all_input_sources(
        properties: Option<CFDictionaryRef>,
        include_all_installed: bool,
    ) -> Option<Vec<TISInputSource>> {
        let properties = match properties {
            Some(properties) => properties,
            None => ptr::null(),
        };

        unsafe {
            let sources = TISCreateInputSourceList(properties, include_all_installed);
            if sources.is_null() {
                return None;
            }
            let sources = CFArray::<TISInputSource>::wrap_under_create_rule(sources);

            Some(sources.into_iter().map(|value| value.to_owned()).collect())
        }
    }

    fn register(location: impl AsRef<Path>) -> Result<(), InputMethodError> {
        debug!("Registering input source...");

        let url = match CFURL::from_path(location, true) {
            Some(url) => url,
            None => return Err(InputMethodError::InvalidDestination),
        };

        unsafe {
            match TISRegisterInputSource(url.as_concrete_TypeRef()) {
                0 => Ok(()),
                i => Err(InputMethodError::OSStatusError(i)),
            }
        }
    }

    pub fn list_input_sources_for_bundle_id(bundle_id: &str) -> Option<Vec<TISInputSource>> {
        let key: CFString = unsafe { CFString::wrap_under_create_rule(kTISPropertyBundleID) };
        let value = CFString::from(bundle_id);
        let properties = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);

        InputMethod::list_all_input_sources(Some(properties.as_concrete_TypeRef()), true)
    }
}

extern "C" {
    pub fn CFBundleGetIdentifier(bundle: CFBundleRef) -> CFStringRef;
    pub fn CFPreferencesSynchronize(
        application_id: CFStringRef,
        username: CFStringRef,
        hostname: CFStringRef,
    ) -> Boolean;
    pub static kCFPreferencesCurrentUser: CFStringRef;
    pub static kCFPreferencesCurrentHost: CFStringRef;
}

#[link(name = "AppKit", kind = "framework")]
extern "C" {}

impl InputMethod {
    pub fn input_source(&self) -> Result<TISInputSource, InputMethodError> {
        let bundle_id_string: String = self.bundle_id()?;

        let bundle_identifier = CFString::from(bundle_id_string.as_str());

        unsafe {
            let bundle_id_key: CFString = CFString::wrap_under_get_rule(kTISPropertyBundleID);
            let category_key: CFString = CFString::wrap_under_get_rule(kTISPropertyInputSourceCategory);
            let input_source_key: CFString = CFString::wrap_under_get_rule(kTISPropertyInputSourceID);

            let properties = CFDictionary::from_CFType_pairs(&[
                (bundle_id_key.as_CFType(), bundle_identifier.as_CFType()),
                (
                    category_key.as_CFType(),
                    CFString::from_static_string("TISCategoryPaletteInputSource").as_CFType(),
                ),
                (input_source_key.as_CFType(), bundle_identifier.as_CFType()),
            ]);

            let sources = InputMethod::list_all_input_sources(Some(properties.as_concrete_TypeRef()), true);

            match sources {
                None => Err(InputMethodError::CouldNotListInputSources),
                Some(sources) => {
                    let len = sources.len();
                    match len {
                        0 => Err(InputMethodError::NoInputSourcesForBundleIdentifier(
                            bundle_identifier.to_string().into(),
                        )),
                        1 => Ok(sources.into_iter().next().unwrap()),
                        _len => Ok(sources.into_iter().next().unwrap()), /* Err(InputMethodError::MultipleInputSourcesForBundleIdentifier(len).into()) */
                    }
                },
            }
        }
    }

    fn target_bundle_path(&self) -> Result<PathBuf, InputMethodError> {
        let input_method_name = match self.bundle_path.components().last() {
            Some(name) => name.as_os_str(),
            None => {
                return Err(InputMethodError::InvalidBundle(
                    "Input method bundle name cannot be determined".into(),
                ));
            },
        };

        Ok(InputMethod::input_method_directory().join(input_method_name))
    }

    pub fn bundle_id(&self) -> Result<String, InputMethodError> {
        let url = match CFURL::from_path(&self.bundle_path, true) {
            Some(url) => url,
            None => {
                return Err(InputMethodError::InvalidBundle(
                    "Could not get URL for input method bundle".into(),
                ));
            },
        };

        let bundle = match CFBundle::new(url) {
            Some(bundle) => bundle,
            None => {
                return Err(InputMethodError::InvalidBundle(
                    format!("Could not load bundle for URL {}", self.bundle_path.display()).into(),
                ));
            },
        };

        let identifier = unsafe { CFBundleGetIdentifier(bundle.as_concrete_TypeRef()) };

        if identifier.is_null() {
            return Err(InputMethodError::InvalidBundle("Could find bundle identifier".into()));
        }

        let bundle_identifier = unsafe { CFString::wrap_under_get_rule(identifier) };

        Ok(bundle_identifier.to_string())
    }
}

fn str_to_nsstring(str: &str) -> &Object {
    const UTF8_ENCODING: usize = 4;
    unsafe {
        let ns_string: &mut Object = msg_send![class!(NSString), alloc];
        let ns_string: &mut Object = msg_send![
            ns_string,
            initWithBytes: str.as_ptr()
            length: str.len()
            encoding: UTF8_ENCODING
        ];
        let _: () = msg_send![ns_string, autorelease];
        ns_string
    }
}

impl Integration for InputMethod {
    fn is_installed(&self) -> Result<()> {
        // let attr = fs::metadata(&self.bundle_path)?;
        let destination = self.target_bundle_path()?;

        // check that symlink to input method exists in input_methods_directory
        let symlink = fs::read_link(destination)?;

        // does it point to the correct location
        if symlink != self.bundle_path {
            return Err(InputMethodError::InvalidBundle("Symbolic link is incorrect".into()).into());
        }

        // todo(mschrage): check that the input method is running (NSRunning application)

        // Can we load input source?
        let input_source = self.input_source()?;

        // Is input source enabled?
        if !input_source.is_enabled().unwrap_or_default() {
            return Err(InputMethodError::NotEnabled.into());
        }

        if !input_source.is_selected().unwrap_or_default() {
            return Err(InputMethodError::NotSelected.into());
        }

        Ok(())
    }

    fn install(&self, _backup_dir: Option<&Path>) -> Result<()> {
        let destination = self.target_bundle_path()?;

        // Attempt to emove existing symlink
        fs::remove_file(&destination).ok();

        // Create new symlink
        symlink(&self.bundle_path, &destination)?;

        // Register input source
        InputMethod::register(&destination)?;

        let input_source = self.input_source()?;

        debug!("Launch Input Method...");
        if let Some(dest) = destination.to_str() {
            Command::new("open").arg(dest);
        }

        // Enable input source. This will prompt user in System Preferences.
        input_source.enable()?;

        // The 'enabled' property of an input source is never updated for the process that invokes
        // `TISEnableInputSource` Unclear why this is, but we handle it by calling out to the
        // fig_cli to finish the second half of the installation.

        // todo: pull this into a function in fig_directories
        let fig_cli_path = match fig_util::fig_bundle() {
            Some(bundle) => bundle.join("Contents").join("MacOS").join(FIG_CLI_BINARY_NAME),
            None => return Err(InputMethodError::HelperExecutableNotFound.into()),
        };

        loop {
            let out = Command::new(fig_cli_path.to_str().expect("Fig CLI can be converted to string"))
                .args([
                    "_",
                    "attempt-to-finish-input-method-installation",
                    self.bundle_path.to_str().unwrap(),
                ])
                .output()
                .unwrap();

            let code = out.status.code().expect("Status code should exist");

            if code == 0 {
                info!("Input method installed successfully!");
                break;
            } else {
                let err = String::from_utf8(out.stdout).unwrap();
                match serde_json::from_str::<InputMethodError>(err.as_str()).ok() {
                    Some(error) => debug!("{error}"),
                    None => debug!("Could not parse output as known error: {err}"),
                }
            }

            std::thread::sleep(std::time::Duration::from_secs(1))
        }

        // TODO: Store PIDs of all relevant terminal emulators (input method will not work until these
        // processes are restarted)

        // NSWorkspace.shared.runningApplications
        // filter based on Terminals with bundle id

        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        let destination = self.target_bundle_path()?;

        let input_source = self.input_source()?;
        input_source.deselect()?;
        input_source.disable()?;

        // todo(mschrage): Terminate input method binary using Cocoa APIs
        let binding = input_source.bundle_id().unwrap();
        unsafe {
            let bundle_id: &Object = str_to_nsstring(binding.as_str());
            let running_input_method_array: &mut Object = msg_send![
                class!(NSRunningApplication),
                runningApplicationsWithBundleIdentifier: bundle_id
            ];
            let running_input_method_array_len: u64 = msg_send![running_input_method_array, count];

            if running_input_method_array_len > 0 {
                let running_input_method: &mut Object = msg_send![running_input_method_array, objectAtIndex: 0];

                let _: () = msg_send![running_input_method, terminate];
            }
        }

        // Remove symbolic link
        fs::remove_file(&destination)?;

        Ok(())
    }

    fn describe(&self) -> String {
        "Input Method".into()
    }
}

impl InputMethod {
    // Called from separate process in order to check status of Input Method
    pub fn finish_input_method_installation(bundle_path: Option<PathBuf>) -> Result<(), InputMethodError> {
        let input_method = match bundle_path {
            Some(bundle_path) if bundle_path.is_absolute() => InputMethod { bundle_path },
            Some(_) => return Err(InputMethodError::InvalidBundlePath),
            None => InputMethod::default(),
        };

        let source = input_method.input_source()?;

        if !source.is_enabled().unwrap_or_default() {
            return Err(InputMethodError::NotEnabled);
        }

        source.select()?;

        if !source.is_selected().unwrap_or_default() {
            return Err(InputMethodError::NotSelected);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn input_method() -> TISInputSource {
        let bundle_identifier = "io.fig.caret";

        let key: CFString = unsafe { CFString::wrap_under_create_rule(kTISPropertyBundleID) };
        let value = CFString::from_static_string(bundle_identifier);
        let properties = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
        let sources =
            InputMethod::list_all_input_sources(Some(properties.as_concrete_TypeRef()), true).unwrap_or_default();
        sources.into_iter().next().unwrap()
    }

    const TEST_INPUT_METHOD_BUNDLE_ID: &str = "io.fig.caret.4";
    const TEST_INPUT_METHOD_BUNDLE_URL: &str = "/Users/mschrage/p/macos/fig_input_method/build/FigInputMethod4.app";

    #[ignore]
    #[test]
    fn check_enabled() {
        let method = InputMethod {
            bundle_path: TEST_INPUT_METHOD_BUNDLE_URL.into(),
        };

        println!(
            "{} enabled: {}",
            method.input_source().unwrap().bundle_id().unwrap(),
            method.input_source().unwrap().is_enabled().unwrap()
        )
    }

    #[ignore]
    #[test]
    fn install() {
        let method = InputMethod {
            bundle_path: TEST_INPUT_METHOD_BUNDLE_URL.into(),
        };

        let bundle_id = TEST_INPUT_METHOD_BUNDLE_ID;
        match InputMethod::list_input_sources_for_bundle_id(bundle_id) {
            Some(inputs) => {
                println!("Uninstalling...");
                inputs
                    .iter()
                    .for_each(|s| println!("{}", s.is_enabled().unwrap_or_default()));
                match method.uninstall() {
                    Ok(_) => println!("Uninstalled!"),
                    Err(e) => println!("{e}"),
                }
            },
            None => {
                println!("No input sources found for {}", bundle_id);
                println!("Installing...");
                match method.install(None) {
                    Ok(_) => println!("Installed!"),
                    Err(e) => println!("{e}"),
                };
            },
        }
    }

    #[ignore]
    #[test]
    fn toggle_selection() {
        let source = input_method();
        let selected = source.is_selected();
        match selected {
            Some(true) => {
                source.select().ok();
                assert!(source.is_selected().unwrap_or_default());
                dbg!(source.deselect().ok());
                assert!(!source.is_selected().unwrap_or(true));
                source.select().ok();
                assert!(selected == source.is_selected());
            },
            Some(false) => {
                source.deselect().ok();
                assert!(!source.is_selected().unwrap_or_default());
                source.select().ok();
                assert!(source.is_selected().unwrap_or(false));
                source.deselect().ok();
                assert!(selected == source.is_selected());
            },

            None => unreachable!("Is selected should be defined"),
        }
    }

    #[ignore]
    #[test]
    fn get_input_source_by_bundle_id() {
        let bundle_identifier = TEST_INPUT_METHOD_BUNDLE_ID; //"com.apple.CharacterPaletteIM";
        let sources = InputMethod::list_input_sources_for_bundle_id(bundle_identifier);
        match sources {
            Some(sources) => {
                println!("Found {} matching source", sources.len());
                assert!(sources.len() == 1);
                assert!(sources[0].bundle_id().unwrap() == bundle_identifier);
                assert!(sources[0].category().unwrap() == "TISCategoryPaletteInputSource");

                println!("{:?}", sources[0])
            },
            None => unreachable!("{} should always exist.", bundle_identifier),
        }
    }

    #[ignore]
    #[test]
    fn uninstall_all() {
        let sources = InputMethod::list_input_sources_for_bundle_id(TEST_INPUT_METHOD_BUNDLE_ID).unwrap_or_default();
        sources.iter().for_each(|s| {
            s.deselect().ok();
            s.disable().ok();
        })
    }

    #[ignore]
    #[test]
    fn test_list_all_input_methods() {
        let sources = InputMethod::list_all_input_sources(None, true).unwrap_or_default();

        assert!(!sources.is_empty());

        sources.iter().for_each(|source| source.show());
    }
}
