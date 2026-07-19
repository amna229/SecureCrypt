// Implementación de AlgorithmProvider para criptografía clásica.
//
// Proporciona la configuración TLS necesaria para utilizar
// mecanismos criptográficos clásicos.

use std::sync::Arc;

use crate::crypto::algorithm_provider::AlgorithmProvider;
use rustls::{ClientConfig, ServerConfig};
use rustls::crypto::aws_lc_rs;
use rustls::CipherSuite;
use rustls::NamedGroup;
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

        my_crypto_provider.cipher_suites.retain(|suite| {

            matches!(suite.suite(), CipherSuite::TLS13_AES_128_GCM_SHA256 | CipherSuite::TLS13_AES_256_GCM_SHA384 | CipherSuite::TLS13_CHACHA20_POLY1305_SHA256)

        });


        my_crypto_provider.kx_groups.retain(|group| {

            matches!(group.name(), NamedGroup::secp256r1 | NamedGroup::secp384r1 | NamedGroup::secp521r1 | NamedGroup::X25519 | NamedGroup::X448 | NamedGroup::FFDHE2048 | NamedGroup::FFDHE3072 | NamedGroup::FFDHE4096 | NamedGroup::FFDHE6144 | NamedGroup::FFDHE8192)

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
