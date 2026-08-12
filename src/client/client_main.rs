use std::error::Error;
use tfg_project::client::run_client;
use tfg_project::crypto::crypto_mode::CryptoMode;
use tfg_project::crypto::crypto_selector::get_crypto_provider;



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    println!("Starting client...");

    let crypto_mode = std::env::var("CRYPTO_MODE")
        .unwrap_or_else(|_| "classical".to_string());

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

    let server_addr = std::env::var("SERVER_ADDR")
        .unwrap_or_else(|_| "server:8443".to_string());

    let server_name = std::env::var("SERVER_NAME")
        .unwrap_or_else(|_| "localhost".to_string());

    let crypto_mode = match crypto_mode.as_str() {
        "classical" => CryptoMode::Classical,
        "post_quantum" => CryptoMode::PostQuantum,
        other => {
            return Err(format!("Unknown CRYPTO_MODE: {}", other).into());
        }
    };

    let provider = get_crypto_provider(crypto_mode);

    run_client(
        provider,
        &server_addr,
        &server_name,
        &cipher_suites,
        &kx_groups,
    )
    .await?;

    Ok(())
}