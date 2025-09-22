//! TLS configuration and connector creation.

use crate::ws::error::*;

/// Create TLS connector based on configuration
pub fn create_connector(
    tls_config: &Option<crate::config::TLSConfig>,
) -> WebsocketResult<Option<tokio_tungstenite::Connector>> {
    // If no TLS features are enabled, return an error if config is provided.
    #[cfg(not(any(feature = "websocket-tls-rustls", feature = "websocket-tls-native")))]
    {
        if tls_config.is_some() {
            return Err(WebsocketError::TLSError(
                "TLS configuration provided but no TLS features enabled. Enable either 'websocket-tls-rustls' or 'websocket-tls-native' feature.".to_string()
            ));
        } else {
            return Ok(None);
        }
    }

    // If there are TLS features, apply it to connector.
    #[cfg(any(feature = "websocket-tls-rustls", feature = "websocket-tls-native"))]
    {
        let Some(tls_config) = tls_config else {
            return Ok(None);
        };
        let certificate_data = std::fs::read(&tls_config.certificate)
            .map_err(|e| WebsocketError::TLSError(format!("Failed to read certificate file: {}", e)))?;

        // Determine certificate format based on file extension and content
        let cert_ext = tls_config.certificate
            .extension()
            .and_then(|s| s.to_str());

        #[cfg(feature = "websocket-tls-rustls")]
        {
            let _ = rustls::crypto::CryptoProvider::install_default(
                rustls::crypto::aws_lc_rs::default_provider()
            );
            let certificate = parse_certificate_rustls(&certificate_data, cert_ext)?;
            create_rustls_connector(certificate)
        }

        #[cfg(feature = "websocket-tls-native")]
        {
            let certificate = parse_certificate_native(&certificate_data, cert_ext)?;
            create_native_connector(certificate)
        }
    }
}

#[cfg(feature = "websocket-tls-rustls")]
fn parse_certificate_rustls(
    certificate_data: &[u8],
    ext: Option<&str>,
) -> WebsocketResult<rustls_pki_types::CertificateDer<'static>> {
    use rustls_pki_types::CertificateDer;

    match ext {
        Some("pem") => parse_pem_rustls(certificate_data),
        Some("der") => Ok(CertificateDer::from(certificate_data.to_vec())),
        Some("crt") => {
            // .crt files can be either PEM or DER
            if certificate_data.starts_with(b"-----BEGIN") {
                parse_pem_rustls(certificate_data)
            } else {
                Ok(CertificateDer::from(certificate_data.to_vec()))
            }
        }
        _ => {
            // Try PEM first, fallback to DER
            parse_pem_rustls(certificate_data)
                .or_else(|_| Ok(CertificateDer::from(certificate_data.to_vec())))
        }
    }
}

#[cfg(feature = "websocket-tls-rustls")]
fn parse_pem_rustls(data: &[u8]) -> WebsocketResult<rustls_pki_types::CertificateDer<'static>> {
    rustls_pemfile::certs(&mut data.as_ref())
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| WebsocketError::TLSError(format!("Failed to parse PEM certificate: {}", e)))?
        .into_iter()
        .next()
        .ok_or_else(|| WebsocketError::TLSError("No certificate found in PEM file".to_string()))
}

#[cfg(feature = "websocket-tls-rustls")]
fn create_rustls_connector(
    certificate: rustls_pki_types::CertificateDer<'static>,
) -> WebsocketResult<Option<tokio_tungstenite::Connector>> {
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add(certificate)
        .map_err(|e| WebsocketError::TLSError(format!("Failed to add certificate to root store: {}", e)))?;

    let tls_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(Some(tokio_tungstenite::Connector::Rustls(
        std::sync::Arc::new(tls_config),
    )))
}

#[cfg(feature = "websocket-tls-native")]
fn parse_certificate_native(
    certificate_data: &[u8],
    ext: Option<&str>,
) -> WebsocketResult<native_tls::Certificate> {
    match ext {
        Some("pem") => native_tls::Certificate::from_pem(certificate_data)
            .map_err(|e| WebsocketError::TLSError(format!("Failed to parse PEM certificate: {}", e))),
        Some("der") => native_tls::Certificate::from_der(certificate_data)
            .map_err(|e| WebsocketError::TLSError(format!("Failed to parse DER certificate: {}", e))),
        Some("crt") => {
            // .crt files can be either PEM or DER
            if certificate_data.starts_with(b"-----BEGIN") {
                native_tls::Certificate::from_pem(certificate_data)
                    .map_err(|e| WebsocketError::TLSError(format!("Failed to parse PEM certificate: {}", e)))
            } else {
                native_tls::Certificate::from_der(certificate_data)
                    .map_err(|e| WebsocketError::TLSError(format!("Failed to parse DER certificate: {}", e)))
            }
        }
        _ => {
            // Try PEM first, fallback to DER
            native_tls::Certificate::from_pem(certificate_data)
                .or_else(|_| native_tls::Certificate::from_der(certificate_data))
                .map_err(|e| WebsocketError::TLSError(format!("Failed to parse certificate: {}", e)))
        }
    }
}

#[cfg(feature = "websocket-tls-native")]
fn create_native_connector(
    certificate: native_tls::Certificate,
) -> WebsocketResult<Option<tokio_tungstenite::Connector>> {
    let mut builder = native_tls::TlsConnector::builder();
    builder.add_root_certificate(certificate);

    let tls_connector = builder.build()
        .map_err(|e| WebsocketError::TLSError(format!("Failed to build TLS connector: {}", e)))?;

    Ok(Some(tokio_tungstenite::Connector::NativeTls(tls_connector)))
}