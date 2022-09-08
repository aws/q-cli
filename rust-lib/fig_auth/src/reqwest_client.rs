use std::env::current_exe;
use std::path::Path;
use std::time::Duration;

use once_cell::sync::Lazy;
use reqwest::{
    Certificate,
    Client,
};

pub static CLIENT: Lazy<Option<Client>> = Lazy::new(|| {
    let danger_accept_invalid_certs = std::env::var_os("FIG_DANGER_ACCEPT_INVALID_CERTS").is_some()
        || fig_settings::state::get_bool_or("FIG_DANGER_ACCEPT_INVALID_CERTS", false);
    let custom_cert = std::env::var("FIG_CUSTOM_CERT")
        .ok()
        .or_else(|| fig_settings::state::get_string("FIG_CUSTOM_CERT").ok().flatten());

    let mut name = current_exe()
        .ok()
        .and_then(|exe| exe.file_stem().and_then(|name| name.to_str().map(String::from)))
        .unwrap_or_else(|| "rust-client".into());

    if name == "fig" || name == "fig-darwin-universal" {
        if let Some(arg1) = std::env::args().nth(1) {
            if arg1 == "_" {
                if let Some(arg2) = std::env::args().nth(2) {
                    name = format!("fig-internal-{arg2}");
                }
            } else if !arg1.starts_with('-') {
                name = format!("fig-{arg1}");
            }
        }
    }

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let version = fig_util::manifest::version().unwrap_or("unknown-version");

    let app_name: String = format!("{name}-{os}-{arch}-{version}")
        .chars()
        .filter(|c| c.is_ascii_graphic())
        .collect();

    let mut client = Client::builder()
        .danger_accept_invalid_certs(danger_accept_invalid_certs)
        .user_agent(app_name)
        .cookie_store(true)
        .timeout(Duration::from_secs(30));

    if let Some(custom_cert) = custom_cert {
        let path = Path::new(&custom_cert);
        if path.exists() {
            match std::fs::read(path) {
                Ok(file) => {
                    let cert = match path.extension().and_then(|e| e.to_str()) {
                        Some("der") => match Certificate::from_der(&file) {
                            Ok(cert) => Some(cert),
                            Err(err) => {
                                tracing::error!(%err, "Failed to deser der file");
                                None
                            },
                        },
                        Some(_) => match Certificate::from_pem(&file) {
                            Ok(cert) => Some(cert),
                            Err(err) => {
                                tracing::error!(%err, "Failed to deser pem file");
                                None
                            },
                        },
                        _ => None,
                    };

                    match cert {
                        Some(cert) => {
                            client = client.add_root_certificate(cert);
                        },
                        None => tracing::error!(?path, "Failed to deser cert"),
                    }
                },
                Err(err) => tracing::error!(%err, ?path, "Failed to read cert file"),
            }
        } else {
            tracing::error!(?path, "Cert path does not exist");
        }
    }

    client.build().ok()
});
