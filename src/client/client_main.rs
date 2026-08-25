use std::error::Error;

use secure_crypt::client::run_client;
use secure_crypt::crypto::{CryptoConfig, CryptoMode};
use uuid::Uuid;

/// Entry point of the SecureCrypt client.
///
/// It reads the evaluation configuration from environment variables,
/// creates the corresponding SecureCrypt configuration and starts the client.
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
        .filter(|value| !value.is_empty())
        .map(String::from)
        .collect();

    let kx_groups: Vec<String> = std::env::var("KX_GROUPS")
        .unwrap_or_default()
        .split(',')
        .filter(|value| !value.is_empty())
        .map(String::from)
        .collect();

    let server_addr = std::env::var("SERVER_ADDR").unwrap_or_else(|_| "server:8443".to_string());

    let server_name = std::env::var("SERVER_NAME").unwrap_or_else(|_| "localhost".to_string());

    let config_url = std::env::var("CONFIG_URL").expect("CONFIG_URL not set");

    let evaluation_started_at =
        std::env::var("EVALUATION_STARTED_AT").expect("EVALUATION_STARTED_AT not set");

    let crypto_config = CryptoConfig::new(CryptoMode::new(crypto_mode), cipher_suites, kx_groups);

    run_client(
        crypto_config,
        &server_addr,
        &server_name,
        &config_url,
        &evaluation_started_at,
        client_id,
        evaluation_id,
    )
    .await?;

    Ok(())
}
