// Implementación de AlgorithmProvider para criptografía postcuántica.
//
// Proporciona la configuración TLS necesaria para utilizar
// mecanismos criptográficos postcuánticos.

use std::sync::Arc;
use crate::crypto::algorithm_provider::AlgorithmProvider;
use rustls::{ClientConfig, ServerConfig};
use rustls::crypto::aws_lc_rs;
use crate::crypto::profiles::cipher_suites;
use crate::crypto::profiles::post_quantum::kx_groups;
use rustls::version;
use rustls::RootCertStore;
use rustls::pki_types::CertificateDer;
use std::fs::File;
use std::io::BufReader;


pub struct PostQuantumProvider;

impl PostQuantumProvider {

    pub fn new() -> Self {
        PostQuantumProvider

    }
}

impl AlgorithmProvider for PostQuantumProvider {
    

    fn build_crypto_provider(&self, selected_cipher_suites: &[String], selected_kx_groups: &[String]) -> rustls::crypto::CryptoProvider {

        // let mut my_crypto_provider = aws_lc_rs::default_provider();

        // let supported_cipher_suites = cipher_suites::supported_cipher_suites();

        // my_crypto_provider.cipher_suites.retain(|suite| {

        //     supported_cipher_suites.contains(&suite.suite())

        // });


        // let supported_kx_groups = kx_groups::supported_kx_groups();

        // my_crypto_provider.kx_groups = supported_kx_groups;

        // my_crypto_provider


        let mut my_crypto_provider = aws_lc_rs::default_provider();

        let supported_cipher_suites = cipher_suites::supported_cipher_suites();

        my_crypto_provider.cipher_suites.retain(|suite| {

            supported_cipher_suites.contains(&suite.suite()) && selected_cipher_suites.iter().any(|selected| {

                selected == &format!("{:?}", suite.suite())

            })

        });


        let supported_kx_groups = kx_groups::supported_kx_groups();

        my_crypto_provider.kx_groups = supported_kx_groups.into_iter().filter(|group| { 
            
            selected_kx_groups.iter().any(|selected| {

                selected == &format!("{:?}", group.name())

            })

        })
        .collect();


        my_crypto_provider

    }


    fn build_client_config(&self, selected_cipher_suites: &[String], selected_kx_groups: &[String]) -> Result<ClientConfig, Box<dyn std::error::Error>> {

        let my_crypto_provider = self.build_crypto_provider(selected_cipher_suites, selected_kx_groups);

        let ca_file = File::open("simplified-pki/rootCA/rootCA.crt")?;
        let mut ca_reader = BufReader::new(ca_file);

        let ca_certificates: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut ca_reader).collect::<Result<Vec<_>, _>>()?;

        let mut root_cert_store = RootCertStore::empty();

        for cert in ca_certificates {

            root_cert_store.add(cert)?;

        }

        let post_quantum_client_config = ClientConfig::builder_with_provider(Arc::new(my_crypto_provider)).with_protocol_versions(&[&version::TLS13])?.with_root_certificates(root_cert_store).with_no_client_auth();

        Ok(post_quantum_client_config)

    }


    fn build_server_config(&self, selected_cipher_suites: &[String], selected_kx_groups: &[String]) -> Result<ServerConfig, Box<dyn std::error::Error>> {

        let my_crypto_provider = self.build_crypto_provider(selected_cipher_suites, selected_kx_groups);

        let cert_file = File::open("simplified-pki/rootCA/server.crt")?;
        let mut cert_reader = BufReader::new(cert_file);
        let cert_chain: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;
        

        let key_file = File::open("simplified-pki/rootCA/private/server.key")?;
        let mut key_reader = BufReader::new(key_file);
        let server_private_key = rustls_pemfile::private_key(&mut key_reader)?.ok_or("Server private key not found")?;


        let post_quantum_server_config = ServerConfig::builder_with_provider(Arc::new(my_crypto_provider)).with_protocol_versions(&[&version::TLS13])?.with_no_client_auth().with_single_cert(cert_chain, server_private_key)?;

        Ok(post_quantum_server_config)

    }


}
