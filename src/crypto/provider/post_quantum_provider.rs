// Implementación de AlgorithmProvider para criptografía postcuántica.
//
// Proporciona la configuración TLS necesaria para utilizar
// mecanismos criptográficos postcuánticos.


use crate::crypto::algorithm_provider::AlgorithmProvider;
use crate::crypto::provider::tls_config;
use rustls::{ClientConfig, ServerConfig};
use rustls::crypto::aws_lc_rs;
use crate::crypto::profiles::cipher_suites;
use crate::crypto::profiles::post_quantum::kx_groups;



pub struct PostQuantumProvider;

impl PostQuantumProvider {

    pub fn new() -> Self {
        PostQuantumProvider

    }
}

impl AlgorithmProvider for PostQuantumProvider {
    

    fn build_crypto_provider(&self, selected_cipher_suites: &[String], selected_kx_groups: &[String]) -> rustls::crypto::CryptoProvider {

        let my_crypto_provider = aws_lc_rs::default_provider();
        let supported_cipher_suites = cipher_suites::supported_cipher_suites();
        let supported_kx_groups = kx_groups::supported_kx_groups();

        tls_config::build_crypto_provider(
            my_crypto_provider,
            supported_cipher_suites,
            supported_kx_groups,
            selected_cipher_suites,
            selected_kx_groups,
        )

    }



    fn build_client_config(&self, selected_cipher_suites: &[String], selected_kx_groups: &[String]) -> Result<ClientConfig, Box<dyn std::error::Error>> {

        let my_crypto_provider = self.build_crypto_provider(selected_cipher_suites, selected_kx_groups);

        tls_config::build_client_config(my_crypto_provider)

    }



    fn build_server_config(&self, selected_cipher_suites: &[String], selected_kx_groups: &[String]) -> Result<ServerConfig, Box<dyn std::error::Error>> {

        let my_crypto_provider = self.build_crypto_provider(selected_cipher_suites, selected_kx_groups);

        tls_config::build_server_config(my_crypto_provider)

    }

}
