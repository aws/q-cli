use std::env::current_exe;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;

use once_cell::sync::Lazy;
use reqwest::Client;
use rustls::client::{
    HandshakeSignatureValid,
    ServerCertVerified,
    ServerCertVerifier,
};
use rustls::{
    ClientConfig,
    DigitallySignedStruct,
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

pub fn create_default_root_cert_store(native_certs: bool) -> RootCertStore {
    let mut root_cert_store = RootCertStore::empty();
    root_cert_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(ta.subject, ta.spki, ta.name_constraints)
    }));

    if native_certs {
        if let Ok(certs) = rustls_native_certs::load_native_certs() {
            for cert in certs {
                // This error is ignored because root certificates often include
                // ancient or syntactically invalid certificates
                root_cert_store.add(&rustls::Certificate(cert.0)).ok();
            }
        }
    }

    let custom_cert = std::env::var("CW_CUSTOM_CERT")
        .ok()
        .or_else(|| fig_settings::state::get_string("CW_CUSTOM_CERT").ok().flatten());

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

// note(grant): we may need to deal with client auth ??
static CLIENT_CONFIG_NATIVE_CERTS: Lazy<Arc<ClientConfig>> = Lazy::new(|| {
    Arc::new(
        ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(create_default_root_cert_store(true))
            .with_no_client_auth(),
    )
});

static CLIENT_CONFIG_NO_NATIVE_CERTS: Lazy<Arc<ClientConfig>> = Lazy::new(|| {
    Arc::new(
        ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(create_default_root_cert_store(false))
            .with_no_client_auth(),
    )
});

static CLIENT_CONFIG_NO_CERTS: Lazy<Arc<ClientConfig>> = Lazy::new(|| {
    Arc::new(
        ClientConfig::builder()
            .with_safe_defaults()
            .with_custom_certificate_verifier(Arc::new(NoVerifier))
            .with_no_client_auth(),
    )
});

pub fn client_config(native_certs: bool) -> Arc<ClientConfig> {
    if std::env::var_os("FIG_DANGER_ACCEPT_INVALID_CERTS").is_some()
        || fig_settings::state::get_bool_or("FIG_DANGER_ACCEPT_INVALID_CERTS", false)
    {
        CLIENT_CONFIG_NO_CERTS.clone()
    } else if native_certs {
        CLIENT_CONFIG_NATIVE_CERTS.clone()
    } else {
        CLIENT_CONFIG_NO_NATIVE_CERTS.clone()
    }
}

pub static USER_AGENT: Lazy<String> = Lazy::new(|| {
    let name = current_exe()
        .ok()
        .and_then(|exe| exe.file_stem().and_then(|name| name.to_str().map(String::from)))
        .unwrap_or_else(|| "unknown-rust-client".into());

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let version = env!("CARGO_PKG_VERSION");

    format!("{name}-{os}-{arch}-{version}")
});

pub fn user_agent() -> &'static str {
    &USER_AGENT
}

pub static CLIENT_NATIVE_CERTS: Lazy<Option<Client>> = Lazy::new(|| {
    Client::builder()
        .use_preconfigured_tls((*client_config(true)).clone())
        .user_agent(USER_AGENT.chars().filter(|c| c.is_ascii_graphic()).collect::<String>())
        .cookie_store(true)
        .build()
        .ok()
});

pub static CLIENT_NO_NATIVE_CERTS: Lazy<Option<Client>> = Lazy::new(|| {
    Client::builder()
        .use_preconfigured_tls((*client_config(false)).clone())
        .user_agent(USER_AGENT.chars().filter(|c| c.is_ascii_graphic()).collect::<String>())
        .cookie_store(true)
        .build()
        .ok()
});

pub fn reqwest_client(native_certs: bool) -> Option<&'static reqwest::Client> {
    if native_certs {
        CLIENT_NATIVE_CERTS.as_ref()
    } else {
        CLIENT_NO_NATIVE_CERTS.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_client() {
        reqwest_client(true).unwrap();
        reqwest_client(false).unwrap();
    }
}
