use crate::crypto::algorithm_provider::AlgorithmProvider;
use crate::crypto::profiles::cipher_suites;
use crate::crypto::profiles::classical::kx_groups;
use crate::crypto::provider::tls_config;
use rustls::crypto::aws_lc_rs;
use rustls::{ClientConfig, ServerConfig};

/// Provides TLS configurations based on classical cryptographic mechanisms.
pub struct ClassicalProvider;

impl ClassicalProvider {
    /// Creates a new classical cryptographic provider.
    pub fn new() -> Self {
        ClassicalProvider
    }
}

impl AlgorithmProvider for ClassicalProvider {
    fn name(&self) -> &'static str {
        "classical"
    }

    /// Builds a crypto provider using the selected classical mechanisms.
    fn build_crypto_provider(
        &self,
        selected_cipher_suites: &[String],
        selected_kx_groups: &[String],
    ) -> rustls::crypto::CryptoProvider {
        let crypto_provider = aws_lc_rs::default_provider();
        let supported_cipher_suites = cipher_suites::supported_cipher_suites();
        let supported_kx_groups = kx_groups::supported_kx_groups();

        tls_config::build_crypto_provider(
            crypto_provider,
            supported_cipher_suites,
            supported_kx_groups,
            selected_cipher_suites,
            selected_kx_groups,
        )
    }

    /// Builds a TLS client configuration using the selected mechanisms.
    fn build_client_config(
        &self,
        selected_cipher_suites: &[String],
        selected_kx_groups: &[String],
    ) -> Result<ClientConfig, Box<dyn std::error::Error + Send + Sync>> {
        let crypto_provider =
            self.build_crypto_provider(selected_cipher_suites, selected_kx_groups);
        tls_config::build_client_config(crypto_provider)
    }

    /// Builds a TLS server configuration using the selected mechanisms.
    fn build_server_config(
        &self,
        selected_cipher_suites: &[String],
        selected_kx_groups: &[String],
    ) -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
        let crypto_provider =
            self.build_crypto_provider(selected_cipher_suites, selected_kx_groups);
        tls_config::build_server_config(crypto_provider)
    }
}





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::algorithm_provider::AlgorithmProvider;

    #[test]
    fn creates_classical_provider() {
        let provider = ClassicalProvider::new();

        assert_eq!(provider.name(), "classical");
    }
}