use crate::crypto::CryptoConfig;

use rustls::pki_types::ServerName;
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{
    TlsAcceptor, TlsConnector, client::TlsStream as ClientTlsStream,
    server::TlsStream as ServerTlsStream,
};

type BoxError = Box<dyn Error + Send + Sync>;

/// Establishes a TLS client connection with the specified server.
///
/// The cryptographic configuration is provided by `CryptoConfig`.
/// The returned TLS stream can be consumed by any application-level
/// protocol.
pub async fn connect_tls(
    crypto_config: &CryptoConfig,
    addr: &str,
    domain: &str,
) -> Result<
    (
        ClientTlsStream<TcpStream>,
        i64,
        Option<String>,
        Option<String>,
    ),
    (BoxError, i64),
> {
    let config = crypto_config
        .build_client_config()
        .map_err(|error| (error, 0))?;

    let connector = TlsConnector::from(Arc::new(config));

    let tcp_stream = TcpStream::connect(addr)
        .await
        .map_err(|error| (Box::new(error) as BoxError, 0))?;

    let server_name = ServerName::try_from(domain.to_string())
        .map_err(|error| (Box::new(error) as BoxError, 0))?;

    let handshake_started_at = Instant::now();

    let tls_stream = connector.connect(server_name, tcp_stream).await;

    let handshake_duration_ms = handshake_started_at.elapsed().as_millis() as i64;

    let tls_stream = match tls_stream {
        Ok(stream) => stream,

        Err(error) => {
            return Err((Box::new(error) as BoxError, handshake_duration_ms));
        }
    };

    let (_, connection) = tls_stream.get_ref();

    let cipher_suite = connection
        .negotiated_cipher_suite()
        .map(|suite| format!("{:?}", suite.suite()));

    let kx_group = connection
        .negotiated_key_exchange_group()
        .map(|group| format!("{:?}", group.name()));

    Ok((tls_stream, handshake_duration_ms, kx_group, cipher_suite))
}

/// Creates a TLS server acceptor from the provided configuration.
///
/// This function is useful when multiple connections need to be
/// accepted using the same TLS configuration.
pub fn create_tls_acceptor(crypto_config: &CryptoConfig) -> Result<TlsAcceptor, BoxError> {
    let config = crypto_config.build_server_config()?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

/// Binds a TCP listener and creates the TLS acceptor used by the server.
///
/// The listener and acceptor can then be used to accept and protect
/// incoming connections independently from the application protocol.
pub async fn create_tls_server(
    crypto_config: &CryptoConfig,
    addr: &str,
) -> Result<(TcpListener, TlsAcceptor), BoxError> {
    let listener = TcpListener::bind(addr).await?;

    let acceptor = create_tls_acceptor(crypto_config)?;

    Ok((listener, acceptor))
}

/// Accepts a TCP connection and establishes TLS using the provided
/// TLS acceptor.
///
/// The resulting TLS stream can be passed to any application-level
/// protocol implementation.
pub async fn accept_tls_connection(
    acceptor: &TlsAcceptor,
    stream: TcpStream,
) -> Result<ServerTlsStream<TcpStream>, BoxError> {
    let tls_stream = acceptor.accept(stream).await?;

    Ok(tls_stream)
}
