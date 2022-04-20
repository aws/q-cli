use std::collections::HashMap;

use crate::{Error, API_DOMAIN, TRACK_SUBDOMAIN};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackEvent {
    RanCommand,
    SelectedShortcut,
    ViaJS,
    UpdatedApp,
    PromptedForAXPermission,
    GrantedAXPermission,
    ToggledAutocomplete,
    ToggledSidebar,
    QuitApp,
    ViewDocs,
    ViewSupportForum,
    JoinSlack,
    SendFeedback,
    DailyAggregates,
    FirstTimeUser,
    ViaShell,
    UninstallApp,
    ITermSetup,
    LaunchedApp,
    FirstAutocompletePopup,
    RestartForOnboarding,
    NewWindowForOnboarding,
    ITermSetupPrompted,
    ShowSecureInputEnabledAlert,
    OpenSecureInputSupportPage,
    OpenedFigMenuIcon,
    InviteAFriend,
    RunInstallationScript,
    TelemetryToggled,
    OpenedSettingsPage,
    DoctorError,
}

impl ToString for TrackEvent {
    fn to_string(&self) -> String {
        match self {
            Self::RanCommand => "Ran CLI command",
            Self::SelectedShortcut => "Selected a Shortcut",
            Self::ViaJS => "Event via JS",
            Self::UpdatedApp => "Updated App",
            Self::PromptedForAXPermission => "Prompted for AX permission",
            Self::GrantedAXPermission => "Granted AX Permission",
            Self::ToggledAutocomplete => "Toggled Autocomplete",
            Self::ToggledSidebar => "Toggled Sidebar",
            Self::QuitApp => "Quit App",
            Self::ViewDocs => "View Docs",
            Self::ViewSupportForum => "View Support Forum",
            Self::JoinSlack => "Join Slack",
            Self::SendFeedback => "Send Feedback",
            Self::DailyAggregates => "Aggregates",
            Self::FirstTimeUser => "First Time User",
            Self::ViaShell => "Event via Shell",
            Self::UninstallApp => "Uninstall App",
            Self::ITermSetup => "iTerm Setup",
            Self::LaunchedApp => "Launched App",
            Self::FirstAutocompletePopup => "First Autocomplete Setup",
            Self::RestartForOnboarding => "Restart for Shell Onboarding",
            Self::NewWindowForOnboarding => "New Window for Shell Onboarding",
            Self::ITermSetupPrompted => "Prompted iTerm Setup",
            Self::ShowSecureInputEnabledAlert => "Show Secure Input Enabled Alert",
            Self::OpenSecureInputSupportPage => "Open Secure Input Support Page",
            Self::OpenedFigMenuIcon => "Opened Fig Menu Icon",
            Self::InviteAFriend => "Prompt to Invite",
            Self::RunInstallationScript => "Running Installation Script",
            Self::TelemetryToggled => "Toggled Telemetry",
            Self::OpenedSettingsPage => "Opened Settings Page",
            Self::DoctorError => "Doctor Error",
        }
        .to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrackSource {
    Cli,
    Daemon,
}

impl std::fmt::Display for TrackSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cli => f.write_str("cli"),
            Self::Daemon => f.write_str("daemon"),
        }
    }
}

pub async fn emit_track<'a, I, T>(
    event: TrackEvent,
    source: TrackSource,
    properties: I,
) -> Result<(), Error>
where
    I: IntoIterator<Item = T>,
    T: Into<(&'a str, &'a str)>,
{
    if fig_settings::settings::get_bool("telemetry.disabled")
        .ok()
        .flatten()
        .unwrap_or(false)
    {
        return Err(Error::TelemetryDisabled);
    }

    // Initial properties
    let mut track = HashMap::from([
        ("userId".into(), fig_auth::get_default("uuid")?),
        ("event".into(), event.to_string()),
    ]);

    // Default properties
    if let Some(email) = fig_auth::get_email() {
        if let Some(domain) = email.split("@").last() {
            track.insert("prop_domain".into(), domain.into());
        }
        track.insert("prop_email".into(), email);
    }

    if let Ok(version) = fig_auth::get_default("versionAtPreviousLaunch") {
        if let Some((version, build)) = version.split_once(',') {
            track.insert("prop_version".into(), version.into());
            track.insert("prop_build".into(), build.into());
        }
    }

    track.insert("prop_source".into(), source.to_string());

    track.insert(
        "install_method".into(),
        crate::install_method::get_install_method().to_string(),
    );

    // Given properties
    for kv in properties.into_iter() {
        let (key, value) = kv.into();
        track.insert(format!("prop_{key}"), value.into());
    }

    // Emit it!
    reqwest::Client::new()
        .post(format!("{}{}", API_DOMAIN, TRACK_SUBDOMAIN))
        .header("Content-Type", "application/json")
        .json(&track)
        .send()
        .await?;

    Ok(())
}
