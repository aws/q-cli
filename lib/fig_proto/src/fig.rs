pub use crate::proto::fig::*;

impl serde::Serialize for NotificationType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            NotificationType::All => "all",
            NotificationType::NotifyOnEditbuffferChange => "editbuffer_change",
            NotificationType::NotifyOnSettingsChange => "settings_change",
            NotificationType::NotifyOnPrompt => "prompt",
            NotificationType::NotifyOnLocationChange => "location_change",
            NotificationType::NotifyOnProcessChanged => "process_change",
            NotificationType::NotifyOnKeybindingPressed => "keybinding_pressed",
            NotificationType::NotifyOnFocusChanged => "focus_change",
            NotificationType::NotifyOnHistoryUpdated => "history_update",
            NotificationType::NotifyOnApplicationUpdateAvailable => "application_update_available",
            NotificationType::NotifyOnLocalStateChanged => "local_state_change",
            NotificationType::NotifyOnEvent => "event",
        })
    }
}
