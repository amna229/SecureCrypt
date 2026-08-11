use std::sync::Arc;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use crate::client::run_client;
use crate::crypto::crypto_selector::get_crypto_provider;
use crate::dashboard::state::{
    ClientConfig,
    DashboardState,
    ServerConfig,
};
use crate::server::run_server;



pub struct Manager {

    state: Arc<DashboardState>

}



impl Manager {

    pub fn new(state: Arc<DashboardState>) -> Self {

        Self { state }

    }



    pub async fn start(&self) -> Result<(), String>{

        let mut is_running = self.state.is_evaluation_running.lock().await;
        if *is_running {

            return Err("An evaluation is already running".to_string());

        }
        *is_running = true;
        drop(is_running);

        let server_config = self.get_server_config().await?;

        let client_configs = self.get_client_configs().await?;

        let cancellation_token = CancellationToken::new();

        self.save_cancellation_token(cancellation_token.clone()).await;

        if let Err(e) = self.start_server(&server_config, cancellation_token).await {

            let mut is_running = self.state.is_evaluation_running.lock().await;
            *is_running = false;
            return Err(e);
        }

        self.start_clients(&client_configs);

        println!("Server and clients started successfully");

        Ok(())
    }



    async fn get_server_config(&self) -> Result<ServerConfig, String>{

        let config = self.state.server_config.lock().await;

        config.clone().ok_or_else(|| {"Server configuration not found".to_string()})
    }



    async fn get_client_configs(&self) -> Result<Vec<ClientConfig>, String>{

        let configs = self.state.client_configs.lock().await;

        if configs.is_empty() {

            return Err("No client configuration found".to_string());

        }

        Ok(configs.clone())
    }



    async fn save_cancellation_token(&self, cancellation_token: CancellationToken){

        let mut token = self.state.server_cancellation_token.lock().await;

        *token = Some(cancellation_token);
    }



    async fn start_server(&self, config: &ServerConfig, cancellation_token: CancellationToken) -> Result<(), String>{

        println!("Starting server...");

        let provider = get_crypto_provider(config.key_exchange);

        let cipher_suites = config.cipher_suites.clone();

        let kx_groups = config.kx_groups.clone();

        let (ready_sender, ready_receiver) = oneshot::channel();

        tokio::spawn(async move {

            if let Err(error) =
                run_server(
                    provider,
                    "0.0.0.0:8443",
                    cancellation_token,
                    &cipher_suites,
                    &kx_groups,
                    ready_sender,
                ).await
            {
                eprintln!("Error running server: {}", error);
            }

        });

        if ready_receiver.await.is_err() {

            return Err("Server failed to start".to_string());

        }

        let mut application_running = self.state.application_running.lock().await;
        *application_running = true;

        Ok(())

    }



    fn start_clients(&self, configs: &[ClientConfig]){

        for config in configs {

            self.start_client_configuration(config);

        }

    }



    fn start_client_configuration(&self, config: &ClientConfig){

        let cipher_suites = config.cipher_suites.clone();

        let kx_groups = config.kx_groups.clone();

        for client_id in 0..config.num_connections {

            self.start_client(
                client_id,
                config.key_exchange,
                cipher_suites.clone(),
                kx_groups.clone(),
            );

        }

    }



    fn start_client(
        &self,
        client_id: u32,
        key_exchange: crate::crypto::crypto_mode::CryptoMode,
        cipher_suites: Vec<String>,
        kx_groups: Vec<String>,
    ){

        println!("Starting client {}...", client_id + 1);

        tokio::spawn(async move {

        let provider = get_crypto_provider(key_exchange);

        if let Err(error) =
                run_client(
                    provider,
                    "0.0.0.0:8443",
                    "localhost",
                    &cipher_suites,
                    &kx_groups,
                ).await
            {
                eprintln!(
                    "Error in client {}: {}",
                    client_id + 1,
                    error
                );
            }

        });

    }



    pub async fn stop(&self){

        let mut token = self.state.server_cancellation_token.lock().await;

        if let Some(cancellation_token) = token.take() {

            cancellation_token.cancel();

            let mut is_running = self.state.is_evaluation_running.lock().await;
            *is_running = false;

            println!("Server stopped successfully");

        } else {

            println!("No server is currently running");

        }

    }

}