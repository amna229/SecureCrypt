use rustls::{ClientConfig, ServerConfig};

pub trait AlgorithmProvider: Send {

    fn build_crypto_provider(&self, selected_cipher_suites: &[String], selected_kx_groups: &[String]) -> rustls::crypto::CryptoProvider;
    fn build_client_config(&self, selected_cipher_suites: &[String], selected_kx_groups: &[String]) -> Result<ClientConfig, Box<dyn std::error::Error + Send + Sync>>;
    fn build_server_config(&self, selected_cipher_suites: &[String], selected_kx_groups: &[String]) -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>>;
    
}