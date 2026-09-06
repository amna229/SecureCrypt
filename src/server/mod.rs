//! SecureCrypt server.
//!
//! This module provides the server-side interface for applications using
//! SecureCrypt. It is responsible for obtaining the evaluation configuration,
//! creating the TLS acceptor, accepting incoming TCP connections and
//! establishing TLS before exposing the resulting stream to the application
//! protocol.

use crate::control::wait_for_configuration;
use crate::crypto::tls::{accept_tls_connection, create_tls_acceptor};
use crate::crypto::{CryptoConfig, CryptoMode};
use crate::user_application::BoxedApplicationStream;

use std::env;
use std::future::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio_rustls::TlsAcceptor;
use tokio_util::sync::CancellationToken;

pub mod http;
pub mod routes;

/// Starts the SecureCrypt server.
///
/// SecureCrypt handles the cryptographic configuration, TCP listener,
/// TLS acceptor and TLS handshakes. Once a TLS connection is established,
/// the resulting stream is exposed to the application through the
/// `BoxedApplicationStream` abstraction.
pub async fn run_server<F, Fut>(
    crypto_config: CryptoConfig,
    addr: &str,
    cancellation_token: CancellationToken,
    ready_sender: oneshot::Sender<()>,
    connection_handler: Arc<F>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: Fn(BoxedApplicationStream) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
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
                    if let Err(error) = handle_connection(
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

/// Establishes TLS for an incoming connection and passes the resulting
/// stream to the application.
async fn handle_connection<F, Fut>(
    stream: TcpStream,
    client_addr: std::net::SocketAddr,
    acceptor: TlsAcceptor,
    connection_handler: Arc<F>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: Fn(BoxedApplicationStream) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
{
    let tls_stream = accept_tls_connection(&acceptor, stream).await?;

    println!("Server: TLS connection established with {}", client_addr);

    connection_handler(Box::pin(tls_stream)).await?;

    Ok(())
}

/// Starts an application over a SecureCrypt TLS server connection.
///
/// SecureCrypt waits for the evaluation configuration provided by the
/// Dashboard, configures the TLS layer and starts listening on the
/// configured address.
///
/// Each established TLS connection is exposed to the application through
/// the `BoxedApplicationStream` abstraction. The application is responsible
/// only for its application-level protocol.
pub async fn start_application<F, Fut>(
    application: Arc<F>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    F: Fn(BoxedApplicationStream) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<(), Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
{
    let control_addr =
        env::var("SECURECRYPT_CONTROL_ADDR").unwrap_or_else(|_| "0.0.0.0:9090".to_string());

    let config = crate::control::wait_for_configuration(&control_addr).await?;

    let listen_addr = config
        .listen_addr
        .unwrap_or_else(|| "0.0.0.0:8443".to_string());

    let crypto_config = CryptoConfig::new(
        CryptoMode::new(config.crypto_mode),
        config.cipher_suites,
        config.kx_groups,
    );

    let cancellation_token = CancellationToken::new();

    let (ready_sender, _ready_receiver) = oneshot::channel();

    run_server(
        crypto_config,
        &listen_addr,
        cancellation_token,
        ready_sender,
        application,
    )
    .await
}
