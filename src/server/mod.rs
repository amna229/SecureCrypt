use crate::crypto::CryptoConfig;
use crate::crypto::tls::{accept_tls_connection, create_application_stream, create_tls_acceptor};

use std::sync::Arc;

use tokio::io::DuplexStream;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio_rustls::{TlsAcceptor, server::TlsStream};
use tokio_util::sync::CancellationToken;

pub mod http;
pub mod routes;

/// Starts the SecureCrypt server.
///
/// The server obtains its TLS acceptor from the SecureCrypt cryptographic
/// configuration, listens for incoming TCP connections and establishes
/// a TLS connection with each client.
///
/// Once TLS has been established, the application receives a `DuplexStream`
/// through which it can implement its own application-level protocol.
pub async fn run_server<F, Fut>(
    crypto_config: CryptoConfig,
    addr: &str,
    cancellation_token: CancellationToken,
    ready_sender: oneshot::Sender<()>,
    connection_handler: Arc<F>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: Fn(DuplexStream) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    let tls_acceptor = create_tls_acceptor(&crypto_config)?;

    let listener = TcpListener::bind(addr).await?;

    println!("SecureCrypt server up on {}", addr);

    let _ = ready_sender.send(());

    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                println!("Server shutting down..");
                break Ok(());
            }

            result = listener.accept() => {
                let (stream, client_addr) = result?;

                let acceptor = tls_acceptor.clone();

                let connection_handler = Arc::clone(&connection_handler);

                tokio::spawn(async move {
                    if let Err(error) =
                        handle_connection(
                            stream,
                            client_addr,
                            acceptor,
                            connection_handler,
                        )
                        .await
                    {
                        eprintln!("Error handling connection from {}: {}", client_addr, error);
                    }
                });
            }
        }
    }
}

/// Establishes TLS and exposes the resulting connection
/// to the application as a duplex stream.
///
/// SecureCrypt handles the TLS layer, while the application
/// handles the application-level protocol.
async fn handle_connection<F, Fut>(
    stream: TcpStream,
    client_addr: std::net::SocketAddr,
    acceptor: TlsAcceptor,
    connection_handler: Arc<F>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: Fn(DuplexStream) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    let tls_stream = accept_tls_connection(&acceptor, stream).await?;

    println!("Server: TLS connection established with {}", client_addr);

    let application_stream = create_application_stream(tls_stream);

    connection_handler(application_stream).await?;

    Ok(())
}
