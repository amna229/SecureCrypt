use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use crate::client::run_client;
use crate::crypto::crypto_selector::get_crypto_provider;
use crate::dashboard::state::{
    ClientConfig,
    DashboardState,
    ServerConfig,
};
use bollard::Docker;
use bollard::models::ContainerCreateBody;
use bollard::query_parameters::CreateContainerOptionsBuilder;


pub struct Manager {

    state: Arc<DashboardState>,
    docker: Docker

}



impl Manager {

    pub fn new(state: Arc<DashboardState>) -> Self {

        let docker = Docker::connect_with_local_defaults().expect("Cannot connect to Docker");

        Self { state, docker }

    }



    pub async fn test_docker(&self) -> Result<(), String> {

        let version = self.docker.version().await.map_err(|e| e.to_string())?;
        println!("Docker version: {:?}", version);

        Ok(())
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

        if let Err(e) = self.start_server(&server_config).await {

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



    async fn start_server(&self, config: &ServerConfig) -> Result<(), String>{

        println!("Starting server...");

        self.start_server_container(config).await?;


        let mut application_running = self.state.application_running.lock().await;
        *application_running = true;

        Ok(())

    }



    async fn start_server_container(&self, config: &ServerConfig) -> Result<(), String> {

        let provider = config.key_exchange;
        let cipher_suites = config.cipher_suites.clone();
        let kx_groups = config.kx_groups.clone();

        let env = vec![
            format!("KEY_EXCHANGE={:?}", provider),
            format!("CIPHER_SUITES={}", serde_json::to_string(&cipher_suites).unwrap()),
            format!("KX_GROUPS={}", serde_json::to_string(&kx_groups).unwrap()),
        ];

        let container_config = ContainerCreateBody{image: Some("tfg_project-server:latest".to_string()), env: Some(env), ..Default::default()};

        let options = CreateContainerOptionsBuilder::default().name("tfg-server").build();
        
        self.docker
            .create_container(
                Some(options),
                container_config,
            )
            .await
            .map_err(|e| e.to_string())?;

        self.docker
            .start_container(
                "tfg-server",
                None,
            )
            .await
            .map_err(|e| e.to_string())?;

        println!("Server container started");

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



    pub async fn stop(&self) -> Result<(), String> {

        self.docker
            .stop_container("tfg-server", None)
            .await
            .map_err(|e| e.to_string())?;

        let mut is_running = self.state.is_evaluation_running.lock().await;
        *is_running = false;

        let mut application_running = self.state.application_running.lock().await;
        *application_running = false;

        println!("Server container stopped");

        Ok(())

    }

}