use crate::crypto::CryptoConfig;
use crate::crypto::tls::{accept_tls_connection, create_tls_acceptor};

use std::sync::Arc;

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
/// Each established connection is processed independently, allowing
/// multiple clients to communicate with the server concurrently.
///
/// The server can be stopped gracefully through the provided
/// cancellation token.
pub async fn run_server<F, Fut>(
    crypto_config: CryptoConfig,
    addr: &str,
    cancellation_token: CancellationToken,
    ready_sender: oneshot::Sender<()>,
    connection_handler: Arc<F>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: Fn(TlsStream<TcpStream>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    let tls_acceptor = create_tls_acceptor(&crypto_config)?;

    let listener = TcpListener::bind(addr).await?;

    println!("SecureCrypt server up on {}", addr);

    // Notify the caller that the server is ready to accept connections.
    let _ = ready_sender.send(());

    loop {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                println!("Server shutting down..");
                break Ok(());
            }

            result = listener.accept() => {
                let (stream, client_addr) = result?;

                let acceptor =
                    tls_acceptor.clone();

                let connection_handler =
                    Arc::clone(&connection_handler);

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
                        eprintln!(
                            "Error handling connection from {}: {}",
                            client_addr,
                            error
                        );
                    }
                });
            }
        }
    }
}

/// Establishes TLS and processes a client connection.
///
/// The TCP stream is first upgraded to TLS using the configured
/// acceptor. Once the handshake succeeds, the resulting secure
/// connection is passed to the application-level connection handler.
async fn handle_connection<F, Fut>(
    stream: TcpStream,
    client_addr: std::net::SocketAddr,
    acceptor: TlsAcceptor,
    connection_handler: Arc<F>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: Fn(TlsStream<TcpStream>) -> Fut + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>>
        + Send
        + 'static,
{
    let tls_stream = accept_tls_connection(&acceptor, stream).await?;

    println!("Server: TLS connection established with {}", client_addr);

    connection_handler(tls_stream).await?;

    Ok(())
}
