mod application;

use crate::application::db::create_pool;
use crate::application::http::serve_connection;
use std::error::Error;
use std::sync::Arc;
use tfg_project::crypto::crypto_mode::CryptoMode;
use tfg_project::crypto::crypto_selector::get_crypto_provider;
use tfg_project::server::run_server;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Starting application...");

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

    let crypto_mode = CryptoMode::new(crypto_mode);

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

            println!("Application shutting down...");

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
