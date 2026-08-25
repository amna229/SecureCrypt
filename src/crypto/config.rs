//! TLS configuration used by the SecureCrypt cryptographic library.
//!
//! This module represents the cryptographic configuration selected for
//! an evaluation and provides the common entry point for building TLS
//! client and server configurations.

use super::{crypto_mode::CryptoMode, crypto_selector::get_crypto_provider};
use rustls::{ClientConfig, ServerConfig};

/// Cryptographic configuration used to build TLS client and server
/// configurations.
#[derive(Debug, Clone)]
pub struct CryptoConfig {
    /// Selected cryptographic mode.
    pub crypto_mode: CryptoMode,

    /// Cipher suites selected for the evaluation.
    pub cipher_suites: Vec<String>,

    /// Key-exchange groups selected for the evaluation.
    pub kx_groups: Vec<String>,
}

impl CryptoConfig {
    /// Creates a new cryptographic configuration.
    pub fn new(
        crypto_mode: CryptoMode,
        cipher_suites: Vec<String>,
        kx_groups: Vec<String>,
    ) -> Self {
        Self {
            crypto_mode,
            cipher_suites,
            kx_groups,
        }
    }

    /// Builds the Rustls cryptographic provider selected by this configuration.
    pub fn build_crypto_provider(
        &self,
    ) -> Result<Box<dyn crate::crypto::algorithm_provider::AlgorithmProvider>, String> {
        get_crypto_provider(&self.crypto_mode)
    }

    /// Builds a TLS client configuration from the selected
    /// cryptographic settings.
    pub fn build_client_config(
        &self,
    ) -> Result<ClientConfig, Box<dyn std::error::Error + Send + Sync>> {
        let provider = get_crypto_provider(&self.crypto_mode)?;
        provider.build_client_config(&self.cipher_suites, &self.kx_groups)
    }

    /// Builds a TLS server configuration from the selected
    /// cryptographic settings.
    pub fn build_server_config(
        &self,
    ) -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
        let provider = get_crypto_provider(&self.crypto_mode)?;
        provider.build_server_config(&self.cipher_suites, &self.kx_groups)
    }
}
