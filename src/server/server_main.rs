use secure_crypt::crypto::{CryptoConfig, CryptoMode};
use secure_crypt::server::http::serve_connection;
use secure_crypt::server::run_server;

use std::error::Error;
use std::sync::Arc;

use tokio::io::DuplexStream;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

/// Entry point for the SecureCrypt server.
///
/// The server configuration is obtained from environment variables
/// and represented through the SecureCrypt cryptographic configuration.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Starting server...");

    let crypto_mode = std::env::var("CRYPTO_MODE").unwrap_or_else(|_| "classical".to_string());

    let cipher_suites: Vec<String> = std::env::var("CIPHER_SUITES")
        .unwrap_or_default()
        .split(',')
        .filter(|suite| !suite.is_empty())
        .map(String::from)
        .collect();

    let kx_groups: Vec<String> = std::env::var("KX_GROUPS")
        .unwrap_or_default()
        .split(',')
        .filter(|group| !group.is_empty())
        .map(String::from)
        .collect();

    let crypto_config = CryptoConfig::new(CryptoMode::new(crypto_mode), cipher_suites, kx_groups);

    let cancellation_token = CancellationToken::new();

    let (ready_sender, _ready_receiver) = oneshot::channel();

    let connection_handler = Arc::new(|application_stream: DuplexStream| async move {
        serve_connection(application_stream).await
    });

    run_server(
        crypto_config,
        "0.0.0.0:8443",
        cancellation_token,
        ready_sender,
        connection_handler,
    )
    .await?;

    Ok(())
}
