use std::sync::Arc;

use quinn::{ServerConfig, crypto::rustls::QuicServerConfig};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

use crate::error::NetError;

pub fn configure_server(
    cert_chain: Vec<CertificateDer<'static>>,
    key: PrivatePkcs8KeyDer<'static>,
) -> Result<ServerConfig, NetError> {
    let crypto = rustls::ServerConfig::builder_with_provider(Arc::new(
        rustls::crypto::ring::default_provider(),
    ))
    .with_safe_default_protocol_versions()?
    .with_no_client_auth()
    .with_single_cert(cert_chain, PrivateKeyDer::from(key))?;

    Ok(ServerConfig::with_crypto(Arc::new(
        QuicServerConfig::try_from(crypto)?,
    )))
}

#[cfg(feature = "dev")]
pub mod dev {
    // Use self-signed certificates (INSECURE!!) for dev purposes
    use std::sync::Arc;

    use quinn::{ClientConfig, crypto::rustls::QuicClientConfig};
    use rustls::{
        DigitallySignedStruct, SignatureScheme,
        client::danger,
        crypto::{CryptoProvider, verify_tls12_signature, verify_tls13_signature},
        pki_types::{CertificateDer, PrivatePkcs8KeyDer, ServerName, UnixTime},
    };

    use crate::error::NetError;

    #[derive(Debug)]
    struct SkipServerVerification(Arc<CryptoProvider>);

    impl SkipServerVerification {
        fn new() -> Arc<Self> {
            Arc::new(Self(Arc::new(rustls::crypto::ring::default_provider())))
        }
    }

    impl danger::ServerCertVerifier for SkipServerVerification {
        fn verify_server_cert(
            &self,
            _end_entity: &CertificateDer<'_>,
            _intermediates: &[CertificateDer<'_>],
            _server_name: &ServerName<'_>,
            _ocsp: &[u8],
            _now: UnixTime,
        ) -> Result<danger::ServerCertVerified, rustls::Error> {
            Ok(danger::ServerCertVerified::assertion())
        }
        fn verify_tls12_signature(
            &self,
            message: &[u8],
            cert: &CertificateDer<'_>,
            dss: &DigitallySignedStruct,
        ) -> Result<danger::HandshakeSignatureValid, rustls::Error> {
            verify_tls12_signature(
                message,
                cert,
                dss,
                &self.0.signature_verification_algorithms,
            )
        }

        fn verify_tls13_signature(
            &self,
            message: &[u8],
            cert: &CertificateDer<'_>,
            dss: &DigitallySignedStruct,
        ) -> Result<danger::HandshakeSignatureValid, rustls::Error> {
            verify_tls13_signature(
                message,
                cert,
                dss,
                &self.0.signature_verification_algorithms,
            )
        }

        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            self.0.signature_verification_algorithms.supported_schemes()
        }
    }

    pub fn generate_self_signed_cert()
    -> Result<(Vec<CertificateDer<'static>>, PrivatePkcs8KeyDer<'static>), NetError> {
        let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
        let cert_der = CertificateDer::from(cert.cert);
        let key = PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der());
        Ok((vec![cert_der], key))
    }

    pub fn configure_client() -> Result<ClientConfig, NetError> {
        let crypto = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()?
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

        Ok(ClientConfig::new(Arc::new(QuicClientConfig::try_from(
            crypto,
        )?)))
    }
}

#[cfg(not(feature = "dev"))]
pub mod prod {
    use std::{
        fs::File,
        io::{self, BufReader},
        path::Path,
        sync::Arc,
    };

    use quinn::{ClientConfig, crypto::rustls::QuicClientConfig};
    use rustls::pki_types::{CertificateDer, PrivatePkcs8KeyDer};

    use crate::error::NetError;

    pub fn configure_client() -> Result<ClientConfig, NetError> {
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

    pub fn load_certs_from_file(
        cert_path: impl AsRef<Path>,
        key_path: impl AsRef<Path>,
    ) -> Result<(Vec<CertificateDer<'static>>, PrivatePkcs8KeyDer<'static>), NetError> {
        let mut cert_reader = BufReader::new(File::open(cert_path)?);
        let cert_chain = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;
        if cert_chain.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "no certificate found").into());
        }

        let mut key_reader = BufReader::new(File::open(key_path)?);
        let key = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
            .next()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no private key found"))??;

        Ok((cert_chain, key))
    }
}
