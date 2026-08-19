use std::error::Error;
use std::sync::Arc;
use tfg_project::crypto::crypto_mode::CryptoMode;
use tfg_project::crypto::crypto_selector::get_crypto_provider;
use tfg_project::server::http::serve_connection;
use tfg_project::server::run_server;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Starting server...");

    let crypto_mode = std::env::var("CRYPTO_MODE").unwrap_or_else(|_| "classical".to_string());

    let cipher_suites: Vec<String> = std::env::var("CIPHER_SUITES")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    let kx_groups: Vec<String> = std::env::var("KX_GROUPS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    let mode = CryptoMode::new(crypto_mode);

    let provider = get_crypto_provider(&mode)?;

    let cancellation_token = CancellationToken::new();

    let (ready_sender, _ready_receiver) = oneshot::channel();

    let connection_handler =
        Arc::new(|tls_stream| async move { serve_connection(tls_stream).await });

    run_server(
        provider,
        "0.0.0.0:8443",
        cancellation_token,
        &cipher_suites,
        &kx_groups,
        ready_sender,
        connection_handler,
    )
    .await?;

    Ok(())
}
