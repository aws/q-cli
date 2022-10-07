use std::borrow::Cow;
use std::iter::empty;
use std::path::Path;
use std::time::Duration;

use accessibility_sys::{
    AXUIElementCreateSystemWide,
    AXUIElementSetMessagingTimeout,
};
use macos_accessibility_position::accessibility::{
    accessibility_is_enabled,
    open_accessibility,
};
use macos_accessibility_position::{
    NSString,
    NotificationCenter,
};

use super::Integration;
use crate::error::{
    Error,
    Result,
};

pub struct AccessibilityIntegration {}

impl Integration for AccessibilityIntegration {
    fn describe(&self) -> String {
        "MacOS Accessibility Integration".to_owned()
    }

    fn install(&self, _backup_dir: Option<&Path>) -> Result<()> {
        if accessibility_is_enabled() {
            return Ok(());
        }

        open_accessibility();

        tokio::spawn(async move {
            fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
                fig_telemetry::TrackEventType::PromptedForAXPermission,
                fig_telemetry::TrackSource::Desktop,
                env!("CARGO_PKG_VERSION").into(),
                empty::<(&str, &str)>(),
            ))
            .await
            .ok();
        });

        // let (update_tx, update_rx) = tokio::sync::oneshot::channel::<()>;

        let ax_notification_name: NSString = "com.apple.accessibility.api".into();
        NotificationCenter::distributed().subscribe(ax_notification_name, |_, subscription| {
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                if accessibility_is_enabled() {
                    fig_telemetry::emit_track(fig_telemetry::TrackEvent::new(
                        fig_telemetry::TrackEventType::GrantedAXPermission,
                        fig_telemetry::TrackSource::Desktop,
                        env!("CARGO_PKG_VERSION").into(),
                        empty::<(&str, &str)>(),
                    ))
                    .await
                    .ok();

                    unsafe {
                        // This prevents Fig from becoming unresponsive if one of the applications
                        // we are tracking becomes unresponsive.
                        AXUIElementSetMessagingTimeout(AXUIElementCreateSystemWide(), 0.25);
                    }
                    // update_tx.send(());
                    let mut sub = subscription.lock();
                    sub.cancel();
                }
            });
        });

        Ok(())
    }

    fn uninstall(&self) -> Result<()> {
        Ok(())
    }

    fn is_installed(&self) -> Result<()> {
        if accessibility_is_enabled() {
            Ok(())
        } else {
            Err(Error::NotInstalled(Cow::Borrowed(
                "Accessibility permissions are not enabled",
            )))
        }
    }
}
