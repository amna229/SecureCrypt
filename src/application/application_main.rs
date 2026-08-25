//! Application service entry point.
//!
//! This module initializes the application database, configures the
//! selected cryptographic provider, starts the TLS server, and handles
//! graceful application shutdown.

use secure_crypt::application::db::create_pool;
use secure_crypt::application::http::serve_connection;
use secure_crypt::crypto::crypto_mode::CryptoMode;
use secure_crypt::crypto::crypto_selector::get_crypto_provider;
use secure_crypt::server::run_server;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

/// Loads a comma-separated environment variable into a vector of strings.
fn load_list_from_env(name: &str) -> Vec<String> {
    std::env::var(name)
        .unwrap_or_default()
        .split(',')
        .filter(|value| !value.is_empty())
        .map(String::from)
        .collect()
}

/// Loads the cryptographic configuration from environment variables.
fn load_crypto_configuration() -> (CryptoMode, Vec<String>, Vec<String>) {
    let crypto_mode = std::env::var("CRYPTO_MODE").unwrap_or_else(|_| "classical".to_string());

    let cipher_suites = load_list_from_env("CIPHER_SUITES");

    let kx_groups = load_list_from_env("KX_GROUPS");

    (CryptoMode::new(crypto_mode), cipher_suites, kx_groups)
}

/// Starts the application service.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Starting application...");

    let (crypto_mode, cipher_suites, kx_groups) = load_crypto_configuration();

    let provider = get_crypto_provider(&crypto_mode)?;

    let cancellation_token = CancellationToken::new();

    let server_cancellation_token = cancellation_token.clone();

    let (ready_sender, _ready_receiver) = oneshot::channel();

    let pool = create_pool().await?;

    let connection_handler = Arc::new(move |tls_stream| {
        let pool = pool.clone();

        async move { serve_connection(tls_stream, pool).await }
    });

    let mut server_future = Box::pin(run_server(
        provider,
        "0.0.0.0:8443",
        server_cancellation_token,
        &cipher_suites,
        &kx_groups,
        ready_sender,
        connection_handler,
    ));

    let mut server_completed = false;

    let mut server_result: Option<Result<(), Box<dyn Error + Send + Sync>>> = None;

    tokio::select! {
        result = tokio::signal::ctrl_c() => {
            result?;

            println!(
                "Application shutting down..."
            );

            cancellation_token.cancel();
        }

        result = &mut server_future => {
            server_completed = true;
            server_result = Some(result);

            cancellation_token.cancel();
        }
    }

    if !server_completed {
        server_result = Some(server_future.as_mut().await);
    }

    match server_result.expect("server result missing") {
        Ok(()) => Ok(()),
        Err(error) => Err(error),
    }
}
