use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use rustls::CipherSuite;
use rustls::{
    crypto::CryptoProvider,
    pki_types::CertificateDer,
    version,
    ClientConfig,
    RootCertStore,
    ServerConfig,
};
use rustls::crypto::SupportedKxGroup;



const ROOT_CA_PATH: &str = "simplified-pki/rootCA/rootCA.crt";
const SERVER_CERTIFICATE_PATH: &str = "simplified-pki/rootCA/server.crt";
const SERVER_PRIVATE_KEY_PATH: &str = "simplified-pki/rootCA/private/server.key";



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
                .any(|selected| {
                    selected == &format!("{:?}", suite.suite())
                })

    });



    crypto_provider.kx_groups = supported_kx_groups
        .into_iter()
        .filter(|group| {

            selected_kx_groups
                .iter()
                .any(|selected| {

                    selected == &format!("{:?}", group.name())

                })

        })
        .collect();

    crypto_provider

}



pub fn build_client_config(crypto_provider: CryptoProvider) -> Result<ClientConfig, Box<dyn std::error::Error>> {

    let root_cert_store =
        load_root_cert_store()?;


    let client_config =
        ClientConfig::builder_with_provider(
            Arc::new(crypto_provider)
        )
        .with_protocol_versions(&[&version::TLS13])?
        .with_root_certificates(root_cert_store)
        .with_no_client_auth();

    Ok(client_config)

}



pub fn build_server_config(crypto_provider: CryptoProvider) -> Result<ServerConfig, Box<dyn std::error::Error>> {

    let cert_chain =
        load_server_certificate()?;

    let private_key =
        load_server_private_key()?;

    let server_config =
        ServerConfig::builder_with_provider(
            Arc::new(crypto_provider)
        )
        .with_protocol_versions(&[&version::TLS13])?
        .with_no_client_auth()
        .with_single_cert(
            cert_chain,
            private_key,
        )?;

    Ok(server_config)

}



fn load_root_cert_store() -> Result<RootCertStore, Box<dyn std::error::Error>>{

    let ca_file = File::open(ROOT_CA_PATH)?;
    let mut ca_reader = BufReader::new(ca_file);
    let ca_certificates:Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut ca_reader).collect::<Result<Vec<_>, _>>()?;

    let mut root_cert_store = RootCertStore::empty();

    for certificate in ca_certificates {

        root_cert_store.add(certificate)?;

    }

    Ok(root_cert_store)

}



fn load_server_certificate() -> Result<Vec<CertificateDer<'static>>, Box<dyn std::error::Error>>{

    let cert_file = File::open(SERVER_CERTIFICATE_PATH)?;
    let mut cert_reader = BufReader::new(cert_file);

    let cert_chain = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;

    Ok(cert_chain)

}



fn load_server_private_key() -> Result<rustls::pki_types::PrivateKeyDer<'static>, Box<dyn std::error::Error>>{

    let key_file = File::open(SERVER_PRIVATE_KEY_PATH)?;
    let mut key_reader = BufReader::new(key_file);

    let private_key = rustls_pemfile::private_key(&mut key_reader)?.ok_or("Server private key not found")?;

    Ok(private_key)

}