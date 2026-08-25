use std::error::Error;

use secure_crypt::client::run_client;
use secure_crypt::crypto::crypto_mode::CryptoMode;
use secure_crypt::crypto::crypto_selector::get_crypto_provider;
use uuid::Uuid;

/// Entry point of the SecureCrypt client.
///
/// It reads the client configuration from environment variables,
/// selects the required cryptographic provider and starts the client.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Starting client...");

    let client_id: u32 = std::env::var("CLIENT_ID")
        .expect("CLIENT_ID not set")
        .parse()?;

    let evaluation_id: Uuid = std::env::var("EVALUATION_ID")
        .expect("EVALUATION_ID not set")
        .parse()?;

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

    let server_addr = std::env::var("SERVER_ADDR").unwrap_or_else(|_| "server:8443".to_string());

    let server_name = std::env::var("SERVER_NAME").unwrap_or_else(|_| "localhost".to_string());

    let config_url = std::env::var("CONFIG_URL").expect("CONFIG_URL not set");

    let evaluation_started_at =
        std::env::var("EVALUATION_STARTED_AT").expect("EVALUATION_STARTED_AT not set");

    let mode = CryptoMode::new(crypto_mode);

    let provider = get_crypto_provider(&mode)?;

    run_client(
        provider,
        &server_addr,
        &server_name,
        &cipher_suites,
        &kx_groups,
        &config_url,
        &evaluation_started_at,
        client_id,
        evaluation_id,
    )
    .await?;

    Ok(())
}
