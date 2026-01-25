use anyhow::{Context, Result};
use axum_server::tls_rustls::RustlsConfig;
use rustls::pki_types::CertificateDer;
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use tracing::info;

use crate::config::TlsConfig;

pub async fn load_tls_config(config: &TlsConfig) -> Result<RustlsConfig> {
    let cert_path = config
        .cert_path
        .as_ref()
        .context("TLS cert_path is required when TLS is enabled")?;
    let key_path = config
        .key_path
        .as_ref()
        .context("TLS key_path is required when TLS is enabled")?;

    if config.require_client_cert {
        info!("TLS mTLS mode enabled - requiring client certificates");

        let client_ca_path = config
            .client_ca_path
            .as_ref()
            .context("client_ca_path is required when require_client_cert is true")?;

        let ca_file = File::open(client_ca_path).context("Failed to open client CA file")?;
        let mut ca_reader = BufReader::new(ca_file);

        let ca_certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut ca_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("Failed to parse client CA certificate")?;

        let mut root_cert_store = RootCertStore::empty();
        for cert in ca_certs {
            root_cert_store.add(cert)?;
        }

        let client_verifier = WebPkiClientVerifier::builder(Arc::new(root_cert_store))
            .build()
            .context("Failed to build client certificate verifier")?;

        let cert_file = File::open(cert_path).context("Failed to open TLS certificate file")?;
        let mut cert_reader = BufReader::new(cert_file);

        let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("Failed to parse TLS certificate")?;

        let key_file = File::open(key_path).context("Failed to open TLS private key file")?;
        let mut key_reader = BufReader::new(key_file);

        let key = rustls_pemfile::private_key(&mut key_reader)
            .context("Failed to read private key")?
            .context("No private key found in key file")?;

        let mut server_config = ServerConfig::builder()
            .with_client_cert_verifier(client_verifier)
            .with_single_cert(certs, key)
            .context("Failed to build TLS server config with mTLS")?;

        server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        Ok(RustlsConfig::from_config(Arc::new(server_config)))
    } else {
        info!("TLS mode enabled (no client certificate required)");

        RustlsConfig::from_pem_file(cert_path, key_path)
            .await
            .context("Failed to load TLS configuration from PEM files")
    }
}

pub fn build_client_config(config: &TlsConfig) -> Result<rustls::ClientConfig> {
    let mut root_cert_store = RootCertStore::empty();

    root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let mut client_config = if let (Some(cert_path), Some(key_path)) = (
        config.client_cert_path.as_ref(),
        config.client_key_path.as_ref(),
    ) {
        info!("Building client with mTLS support");

        let cert_file = File::open(cert_path).context("Failed to open client certificate file")?;
        let mut cert_reader = BufReader::new(cert_file);

        let certs: Vec<CertificateDer> = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse client certificate")?;

        let key_file = File::open(key_path).context("Failed to open client private key file")?;
        let mut key_reader = BufReader::new(key_file);

        let key = rustls_pemfile::private_key(&mut key_reader)
            .context("Failed to read client private key")?
            .context("No private key found in client key file")?;

        rustls::ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_client_auth_cert(certs, key)
            .context("Failed to build client config with mTLS")?
    } else {
        rustls::ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth()
    };

    client_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

    Ok(client_config)
}
