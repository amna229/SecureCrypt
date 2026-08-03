use std::sync::Arc;

use tokio_util::sync::CancellationToken;

use crate::client::run_client;
use crate::crypto::crypto_selector::get_crypto_provider;
use crate::dashboard::state::{
    ClientConfig,
    DashboardState,
    ServerConfig,
};
use crate::server::run_server;



pub async fn start(state: Arc<DashboardState>) -> Result<(), String> {

    let server_config = get_server_config(&state).await?;
    let client_configs = get_client_configs(&state).await?;

    let cancellation_token = CancellationToken::new();

    save_cancellation_token(&state, cancellation_token.clone()).await;

    start_server(&server_config, cancellation_token).await?;

    start_clients(&client_configs).await;

    Ok(())
}



async fn get_server_config(state: &DashboardState) -> Result<ServerConfig, String> {

    let config = state.server_config.lock().await;

    config.clone().ok_or_else(|| {"Server configuration not found".to_string()})
}



async fn get_client_configs(state: &DashboardState) -> Result<Vec<ClientConfig>, String> {

    let configs = state.client_configs.lock().await;

    if configs.is_empty() {

        return Err("No client configuration found".to_string());

    }

    Ok(configs.clone())
}



async fn save_cancellation_token(state: &DashboardState, cancellation_token: CancellationToken){

    let mut token = state.server_cancellation_token.lock().await;
    *token = Some(cancellation_token);

}



async fn start_server(config: &ServerConfig, cancellation_token: CancellationToken) -> Result<(), String>{

    println!("Starting server...");

    let provider = get_crypto_provider(config.key_exchange);

    let cipher_suites = config.cipher_suites.clone();

    let kx_groups = config.kx_groups.clone();

    tokio::spawn(async move {

        if let Err(error) =
            run_server(
                provider,
                "127.0.0.1:8443",
                cancellation_token,
                &cipher_suites,
                &kx_groups,
            ).await
        {
            eprintln!("Error running server: {}", error);
        }

    });

    Ok(())
}



async fn start_clients(configs: &[ClientConfig]){

    for config in configs {

        start_client_configuration(config).await;

    }

}



async fn start_client_configuration(config: &ClientConfig){

    let cipher_suites = config.cipher_suites.clone();

    let kx_groups = config.kx_groups.clone();

    for client_id in 0..config.num_connections {

        start_client(
            client_id,
            config.key_exchange,
            cipher_suites.clone(),
            kx_groups.clone(),
        ).await;

    }

}



async fn start_client(client_id: u32, key_exchange: crate::crypto::crypto_mode::CryptoMode, cipher_suites: Vec<String>, kx_groups: Vec<String>){

    println!("Starting client {}...", client_id + 1);

    let provider = get_crypto_provider(key_exchange);

    if let Err(error) =
        run_client(
            provider,
            "127.0.0.1:8443",
            "localhost",
            &cipher_suites,
            &kx_groups,
        ).await
    {
        eprintln!("Error in client {}: {}", client_id + 1, error);
    }

}