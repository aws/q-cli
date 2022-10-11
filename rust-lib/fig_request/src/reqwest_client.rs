use std::env::current_exe;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use once_cell::sync::Lazy;
use reqwest::Client;
use rustls::client::{
    HandshakeSignatureValid,
    ServerCertVerified,
    ServerCertVerifier,
};
use rustls::internal::msgs::handshake::DigitallySignedStruct;
use rustls::{
    ClientConfig,
    Error,
    RootCertStore,
    ServerName,
};

// This is very similar to the same struct in reqwest
struct NoVerifier;

impl ServerCertVerifier for NoVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &rustls::Certificate,
        _intermediates: &[rustls::Certificate],
        _server_name: &ServerName,
        _scts: &mut dyn Iterator<Item = &[u8]>,
        _ocsp_response: &[u8],
        _now: std::time::SystemTime,
    ) -> Result<ServerCertVerified, Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::Certificate,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &rustls::Certificate,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }
}

pub fn create_default_root_cert_store() -> RootCertStore {
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_server_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.0.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(ta.subject, ta.spki, ta.name_constraints)
    }));
    if let Ok(certs) = rustls_native_certs::load_native_certs() {
        for cert in certs {
            // This error is ignored because root certificates often include
            // ancient or syntactically invalid certificates
            root_cert_store.add(&rustls::Certificate(cert.0)).ok();
        }
    }

    let custom_cert = std::env::var("FIG_CUSTOM_CERT")
        .ok()
        .or_else(|| fig_settings::state::get_string("FIG_CUSTOM_CERT").ok().flatten());

    if let Some(custom_cert) = custom_cert {
        match File::open(Path::new(&custom_cert)) {
            Ok(file) => {
                let reader = &mut BufReader::new(file);
                match rustls_pemfile::certs(reader) {
                    Ok(certs) => {
                        root_cert_store.add_parsable_certificates(&certs);
                    },
                    Err(err) => tracing::error!(path =% custom_cert, %err, "Failed to parse cert"),
                }
            },
            Err(err) => tracing::error!(path =% custom_cert, %err, "Failed to open cert at"),
        }
    }

    root_cert_store
}

static CLIENT_CONFIG: Lazy<Arc<ClientConfig>> = Lazy::new(|| {
    // note(grant): we may need to deal with client auth ??

    if std::env::var_os("FIG_DANGER_ACCEPT_INVALID_CERTS").is_some()
        || fig_settings::state::get_bool_or("FIG_DANGER_ACCEPT_INVALID_CERTS", false)
    {
        Arc::new(
            ClientConfig::builder()
                .with_safe_defaults()
                .with_custom_certificate_verifier(Arc::new(NoVerifier))
                .with_no_client_auth(),
        )
    } else {
        Arc::new(
            ClientConfig::builder()
                .with_safe_defaults()
                .with_root_certificates(create_default_root_cert_store())
                .with_no_client_auth(),
        )
    }
});

pub fn client_config() -> Arc<ClientConfig> {
    CLIENT_CONFIG.clone()
}

pub static USER_AGENT: Lazy<String> = Lazy::new(|| {
    let mut name = current_exe()
        .ok()
        .and_then(|exe| exe.file_stem().and_then(|name| name.to_str().map(String::from)))
        .unwrap_or_else(|| "unknown-rust-client".into());

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

    format!("{name}-{os}-{arch}-{version}")
});

pub fn user_agent() -> &'static str {
    &USER_AGENT
}

pub static CLIENT: Lazy<Option<Client>> = Lazy::new(|| {
    Client::builder()
        .use_preconfigured_tls((*client_config()).clone())
        .user_agent(USER_AGENT.chars().filter(|c| c.is_ascii_graphic()).collect::<String>())
        .cookie_store(true)
        .timeout(Duration::from_secs(30))
        .build()
        .ok()
});

pub fn reqwest_client() -> Option<&'static reqwest::Client> {
    CLIENT.as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_client() {
        reqwest_client().unwrap();
    }
}
