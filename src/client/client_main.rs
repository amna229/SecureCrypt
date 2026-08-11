use std::error::Error;

use tfg_project::client::run_client;
use tfg_project::crypto::crypto_mode::CryptoMode;
use tfg_project::crypto::crypto_selector::get_crypto_provider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Starting client...");

    let provider = get_crypto_provider(CryptoMode::Classical);

    run_client(
        provider,
        "server:8443",
        "localhost",
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
    )
    .await?;

    Ok(())
}