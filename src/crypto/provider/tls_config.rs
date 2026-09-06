use rustls::CipherSuite;
use rustls::crypto::SupportedKxGroup;
use rustls::{
    ClientConfig, RootCertStore, ServerConfig, crypto::CryptoProvider, pki_types::CertificateDer,
    version,
};
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

const ROOT_CA_PATH: &str = "simplified-pki/rootCA/rootCA.crt";
const SERVER_CERTIFICATE_PATH: &str = "simplified-pki/rootCA/server.crt";
const SERVER_PRIVATE_KEY_PATH: &str = "simplified-pki/rootCA/private/server.key";

/// Filters the crypto provider according to the selected cipher suites and key exchange groups.
pub fn build_crypto_provider(
    mut crypto_provider: CryptoProvider,
    supported_cipher_suites: &[CipherSuite],
    supported_kx_groups: Vec<&'static dyn SupportedKxGroup>,
    selected_cipher_suites: &[String],
    selected_kx_groups: &[String],
) -> CryptoProvider {
    crypto_provider.cipher_suites.retain(|suite| {
        supported_cipher_suites
            .iter()
            .any(|supported| *supported == suite.suite())
            && selected_cipher_suites
                .iter()
                .any(|selected| selected == &format!("{:?}", suite.suite()))
    });

    crypto_provider.kx_groups = supported_kx_groups
        .into_iter()
        .filter(|group| {
            selected_kx_groups
                .iter()
                .any(|selected| selected == &format!("{:?}", group.name()))
        })
        .collect();

    crypto_provider
}

/// Builds a TLS 1.3 client configuration using the provided crypto provider.
pub fn build_client_config(
    crypto_provider: CryptoProvider,
) -> Result<ClientConfig, Box<dyn std::error::Error + Send + Sync>> {
    let root_cert_store = load_root_cert_store()?;

    let client_config = ClientConfig::builder_with_provider(Arc::new(crypto_provider))
        .with_protocol_versions(&[&version::TLS13])?
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    Ok(client_config)
}

/// Builds a TLS 1.3 server configuration using the provided crypto provider.
pub fn build_server_config(
    crypto_provider: CryptoProvider,
) -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
    let cert_chain = load_server_certificate()?;
    let private_key = load_server_private_key()?;

    let server_config = ServerConfig::builder_with_provider(Arc::new(crypto_provider))
        .with_protocol_versions(&[&version::TLS13])?
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)?;

    Ok(server_config)
}

/// Loads the root CA certificates used to authenticate the server.
fn load_root_cert_store() -> Result<RootCertStore, Box<dyn std::error::Error + Send + Sync>> {
    let ca_file = File::open(ROOT_CA_PATH)?;
    let mut ca_reader = BufReader::new(ca_file);

    let ca_certificates: Vec<CertificateDer<'static>> =
        rustls_pemfile::certs(&mut ca_reader).collect::<Result<Vec<_>, _>>()?;

    let mut root_cert_store = RootCertStore::empty();

    for certificate in ca_certificates {
        root_cert_store.add(certificate)?;
    }

    Ok(root_cert_store)
}

/// Loads the server certificate chain from the configured certificate file.
fn load_server_certificate()
-> Result<Vec<CertificateDer<'static>>, Box<dyn std::error::Error + Send + Sync>> {
    let cert_file = File::open(SERVER_CERTIFICATE_PATH)?;
    let mut cert_reader = BufReader::new(cert_file);

    let cert_chain = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;

    Ok(cert_chain)
}

/// Loads the server private key from the configured key file.
fn load_server_private_key()
-> Result<rustls::pki_types::PrivateKeyDer<'static>, Box<dyn std::error::Error + Send + Sync>> {
    let key_file = File::open(SERVER_PRIVATE_KEY_PATH)?;
    let mut key_reader = BufReader::new(key_file);

    let private_key =
        rustls_pemfile::private_key(&mut key_reader)?.ok_or("Server private key not found")?;

    Ok(private_key)
}





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use rustls::crypto::aws_lc_rs;

    #[test]
    fn selects_requested_classical_cipher_suite() {
        let crypto_provider = aws_lc_rs::default_provider();

        let supported_cipher_suites = crate::crypto::profiles::cipher_suites::supported_cipher_suites();
        let supported_kx_groups =
            crate::crypto::profiles::classical::kx_groups::supported_kx_groups();

        let selected_cipher_suites = vec!["TLS13_AES_128_GCM_SHA256".to_string()];
        let selected_kx_groups = vec!["X25519".to_string()];

        let provider = build_crypto_provider(
            crypto_provider,
            supported_cipher_suites,
            supported_kx_groups,
            &selected_cipher_suites,
            &selected_kx_groups,
        );

        assert!(
            provider
                .cipher_suites
                .iter()
                .any(|suite| format!("{:?}", suite.suite()) == "TLS13_AES_128_GCM_SHA256")
        );

        assert!(
            provider
                .kx_groups
                .iter()
                .any(|group| format!("{:?}", group.name()) == "X25519")
        );
    }

    #[test]
    fn selects_requested_post_quantum_kx_group() {
        let crypto_provider = aws_lc_rs::default_provider();

        let supported_cipher_suites = crate::crypto::profiles::cipher_suites::supported_cipher_suites();
        let supported_kx_groups =
            crate::crypto::profiles::post_quantum::kx_groups::supported_kx_groups();

        let selected_cipher_suites = vec!["TLS13_AES_128_GCM_SHA256".to_string()];
        let selected_kx_groups = vec!["MLKEM768".to_string()];

        let provider = build_crypto_provider(
            crypto_provider,
            supported_cipher_suites,
            supported_kx_groups,
            &selected_cipher_suites,
            &selected_kx_groups,
        );

        assert!(
            provider
                .cipher_suites
                .iter()
                .any(|suite| format!("{:?}", suite.suite()) == "TLS13_AES_128_GCM_SHA256")
        );

        assert!(
            provider
                .kx_groups
                .iter()
                .any(|group| format!("{:?}", group.name()) == "MLKEM768")
        );
    }

    #[test]
    fn unsupported_cipher_suite_is_filtered_out() {
        let crypto_provider = aws_lc_rs::default_provider();

        let supported_cipher_suites = crate::crypto::profiles::cipher_suites::supported_cipher_suites();
        let supported_kx_groups =
            crate::crypto::profiles::classical::kx_groups::supported_kx_groups();

        let selected_cipher_suites = vec!["INVALID_CIPHER_SUITE".to_string()];
        let selected_kx_groups = vec!["X25519".to_string()];

        let provider = build_crypto_provider(
            crypto_provider,
            supported_cipher_suites,
            supported_kx_groups,
            &selected_cipher_suites,
            &selected_kx_groups,
        );

        assert!(
            provider.cipher_suites.is_empty(),
            "Unsupported cipher suites should be filtered out"
        );
    }

    #[test]
    fn unsupported_kx_group_is_filtered_out() {
        let crypto_provider = aws_lc_rs::default_provider();

        let supported_cipher_suites = crate::crypto::profiles::cipher_suites::supported_cipher_suites();
        let supported_kx_groups =
            crate::crypto::profiles::classical::kx_groups::supported_kx_groups();

        let selected_cipher_suites = vec!["TLS13_AES_128_GCM_SHA256".to_string()];
        let selected_kx_groups = vec!["INVALID_KX_GROUP".to_string()];

        let provider = build_crypto_provider(
            crypto_provider,
            supported_cipher_suites,
            supported_kx_groups,
            &selected_cipher_suites,
            &selected_kx_groups,
        );

        assert!(
            provider.kx_groups.is_empty(),
            "Unsupported key exchange groups should be filtered out"
        );
    }
}