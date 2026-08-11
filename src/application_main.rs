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

    let (ready_sender, _ready_receiver) = oneshot::channel();

    run_server(
        provider,
        "0.0.0.0:8443",
        cancellation_token,
        &[
            "TLS13_AES_256_GCM_SHA384".to_string(),
            "TLS13_AES_128_GCM_SHA256".to_string(),
            "TLS13_CHACHA20_POLY1305_SHA256".to_string(),
        ],
        &[
            "secp256r1".to_string(),
            "secp384r1".to_string(),
            "X25519".to_string(),
        ],
        ready_sender,
    )
    .await?;

    Ok(())
}