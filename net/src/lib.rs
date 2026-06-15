use std::{error::Error, sync::Arc};

use quinn::{
    ServerConfig,
    crypto::rustls::{QuicServerConfig},
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

mod cert;

#[cfg(feature = "dev")]
pub use cert::dev::configure_client;

#[cfg(not(feature = "dev"))]
pub use cert::prod::configure_client;
