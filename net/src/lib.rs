use std::{error::Error, sync::Arc};

use quinn::{
    ClientConfig, ServerConfig,
    crypto::rustls::{QuicClientConfig, QuicServerConfig},
};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

pub fn configure_server(
    cert: CertificateDer<'static>,
    key: PrivatePkcs8KeyDer<'static>,
) -> Result<ServerConfig, Box<dyn Error>> {
    let crypto = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_no_client_auth()
    .with_single_cert(vec![cert], PrivateKeyDer::from(key))?;

    Ok(ServerConfig::with_crypto(Arc::new(
        QuicServerConfig::try_from(crypto)?,
    )))
}

pub fn configure_client() -> Result<ClientConfig, Box<dyn Error>> {
    // Use the Mozilla CA root certificate store
    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };

    let crypto = rustls::ClientConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_root_certificates(roots)
    .with_no_client_auth();

    Ok(ClientConfig::new(Arc::new(QuicClientConfig::try_from(
        crypto,
    )?)))
}
