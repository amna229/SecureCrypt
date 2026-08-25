use rustls::{ClientConfig, ServerConfig};

/// Abstraction over the cryptographic mechanisms used by the application.
///
/// Implementations provide the cryptographic configuration required to
/// establish TLS connections in either classical or post-quantum mode.
pub trait AlgorithmProvider: Send {
    /// Returns the identifier of the cryptographic mode.
    fn name(&self) -> &'static str;

    /// Builds a Rustls cryptographic provider using the selected
    /// cipher suites and key exchange groups.
    fn build_crypto_provider(
        &self,
        selected_cipher_suites: &[String],
        selected_kx_groups: &[String],
    ) -> rustls::crypto::CryptoProvider;

    /// Builds the TLS client configuration for the selected
    /// cryptographic mechanisms.
    fn build_client_config(
        &self,
        selected_cipher_suites: &[String],
        selected_kx_groups: &[String],
    ) -> Result<ClientConfig, Box<dyn std::error::Error + Send + Sync>>;

    /// Builds the TLS server configuration for the selected
    /// cryptographic mechanisms.
    fn build_server_config(
        &self,
        selected_cipher_suites: &[String],
        selected_kx_groups: &[String],
    ) -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>>;
}
