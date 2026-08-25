use std::error::Error;

use secure_crypt::client::run_client;
use secure_crypt::crypto::{CryptoConfig, CryptoMode};

/// Entry point of the SecureCrypt client.
///
/// The cryptographic configuration and server connection parameters
/// are obtained from environment variables.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Starting client...");

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

    let crypto_config = CryptoConfig::new(CryptoMode::new(crypto_mode), cipher_suites, kx_groups);

    run_client(crypto_config, &server_addr, &server_name).await?;

    Ok(())
}
