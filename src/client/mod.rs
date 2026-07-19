use tokio::net::TcpStream;
use std::error::Error;
use tokio_rustls::TlsConnector;
use tokio_rustls::client::TlsStream;
use std::sync::Arc;
use rustls::pki_types::ServerName;
use crate::crypto::algorithm_provider::AlgorithmProvider;


async fn connect_to_server_tcp(addr: &str) -> Result<TcpStream, Box<dyn Error>>{
    // Connect to a peer
    let stream = TcpStream::connect(addr).await?;

    Ok(stream)
}



pub async fn establish_tls_connection(provider: Box<dyn AlgorithmProvider>, addr: &str, domain: &str) -> Result<TlsStream<TcpStream>, Box<dyn Error>> {

    let config = provider.build_client_config()?;

    let tls_connector = TlsConnector::from(Arc::new(config));

    let tcp_stream = connect_to_server_tcp(addr).await?;

    let server_name = ServerName::try_from(domain.to_string())?;

    let tls_stream = tls_connector
        .connect(server_name, tcp_stream)
        .await?;

    Ok(tls_stream)
}