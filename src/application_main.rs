mod application;
mod crypto;
mod server;

use std::error::Error;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use crate::crypto::crypto_mode::CryptoMode;
use crate::crypto::crypto_selector::get_crypto_provider;
use crate::server::run_server;




#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    println!("Starting application...");

    let provider = get_crypto_provider(CryptoMode::Classical);

    let cancellation_token = CancellationToken::new();
    let server_cancellation_token = cancellation_token.clone();

    let (ready_sender, _ready_receiver) = oneshot::channel();

    let pool = application::db::create_pool().await?;

    let cipher_suites = vec![
        "TLS13_AES_256_GCM_SHA384".to_string(),
        "TLS13_AES_128_GCM_SHA256".to_string(),
        "TLS13_CHACHA20_POLY1305_SHA256".to_string(),
    ];

    let kx_groups = vec![
        "secp256r1".to_string(),
        "secp384r1".to_string(),
        "X25519".to_string(),
    ];

    let mut server_future = Box::pin(run_server(
        provider,
        "0.0.0.0:8443",
        server_cancellation_token,
        &cipher_suites,
        &kx_groups,
        ready_sender,
        pool
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

    match server_result.expect("server result missing"){

        Ok(()) => Ok(()),
        Err(error) => Err(error),
        
    }
}