// Implementación de AlgorithmProvider para criptografía clásica.
//
// Proporciona la configuración TLS necesaria para utilizar
// mecanismos criptográficos clásicos.

use std::sync::Arc;
use crate::crypto::algorithm_provider::AlgorithmProvider;
use rustls::{ClientConfig, ServerConfig};
use rustls::crypto::aws_lc_rs;
use crate::crypto::profiles::cipher_suites;
use crate::crypto::profiles::classical::kx_groups;
use rustls::version;
use rustls::RootCertStore;


pub struct ClassicalProvider;

//devuelve instancia de ClassicalProvider
impl ClassicalProvider {

    pub fn new() -> Self {

        ClassicalProvider

    }
    
}

impl AlgorithmProvider for ClassicalProvider {

    fn build_client_config(&self) -> Result<ClientConfig, Box<dyn std::error::Error>> {

        let mut my_crypto_provider = aws_lc_rs::default_provider();

        let supported_cipher_suites = cipher_suites::supported_cipher_suites();

        my_crypto_provider.cipher_suites.retain(|suite| {

            supported_cipher_suites.contains(&suite.suite())

        });


        let supported_kx_groups = kx_groups::supported_kx_groups();

        my_crypto_provider.kx_groups.retain(|group| {

            supported_kx_groups.contains(&group.name())

        });


        let mut root_cert_store = RootCertStore::empty();

        root_cert_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let classic_client_config = ClientConfig::builder_with_provider(Arc::new(my_crypto_provider)).with_protocol_versions(&[&version::TLS13])?.with_root_certificates(root_cert_store).with_no_client_auth();

        Ok(classic_client_config)

    }


    fn build_server_config(&self) -> Result<ServerConfig, Box<dyn std::error::Error>> {

        println!("Usando proveedor clásico");
        todo!("Implement the build_server_config method for ClassicalProvider");

    }

}
