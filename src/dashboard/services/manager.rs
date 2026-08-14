use crate::dashboard::state::{ClientConfig, DashboardState, ServerConfig};
use bollard::Docker;
use bollard::models::{
    ContainerCreateBody, EndpointSettings, NetworkConnectRequest, NetworkCreateRequest,
};
use bollard::query_parameters::{CreateContainerOptionsBuilder, RemoveContainerOptionsBuilder};
use std::sync::Arc;



pub struct Manager {
    state: Arc<DashboardState>,
    docker: Docker,
}



impl Manager {

    pub fn new(state: Arc<DashboardState>) -> Self {

        let docker = Docker::connect_with_local_defaults().expect("Cannot connect to Docker");

        Self {state, docker}
    }



    pub async fn start(&self) -> Result<(), String> {

        let mut is_running = self.state.is_evaluation_running.lock().await;

        if *is_running {

            return Err("An evaluation is already running".to_string());

        }

        *is_running = true;
        drop(is_running);

        let server_config = self.get_server_config().await?;
        let client_configs = self.get_client_configs().await?;

        self.create_evaluation_network().await?;

        if let Err(e) = self.start_server(&server_config).await {

            let mut is_running = self.state.is_evaluation_running.lock().await;
            *is_running = false;
            return Err(e);

        }

        self.start_clients(&client_configs).await?;

        println!("Server and clients started successfully");

        Ok(())

    }



    async fn get_server_config(&self) -> Result<ServerConfig, String> {

        let config = self.state.server_config.lock().await;

        config
            .clone()
            .ok_or_else(|| "Server configuration not found".to_string())

    }



    async fn get_client_configs(&self) -> Result<Vec<ClientConfig>, String> {

        let configs = self.state.client_configs.lock().await;

        if configs.is_empty(){

            return Err("No client configuration found".to_string());

        }

        Ok(configs.clone())
    }



    async fn start_server(&self, config: &ServerConfig) -> Result<(), String> {

        println!("Starting server...");

        self.start_server_container(config).await?;

        let mut application_running = self.state.application_running.lock().await;
        *application_running = true;

        Ok(())
    }



    async fn start_server_container(&self, config: &ServerConfig) -> Result<(), String> {
        
        let cipher_suites = config.cipher_suites.join(",");
        let kx_groups = config.kx_groups.join(",");

        let database_url = std::env::var("DATABASE_URL").map_err(|_| "DATABASE_URL not set in dashboard".to_string())?;

        let env = vec![
            format!("CRYPTO_MODE={}", config.key_exchange),
            format!("CIPHER_SUITES={}", cipher_suites),
            format!("KX_GROUPS={}", kx_groups),
            format!("DATABASE_URL={}", database_url)
        ];

        let container_config = ContainerCreateBody{
            image: Some("tfg_project-server:latest".to_string()),
            env: Some(env),
            ..Default::default()
        };

        let options = CreateContainerOptionsBuilder::default()
            .name("tfg-server")
            .build();

        self.docker
            .create_container(Some(options), container_config)
            .await
            .map_err(|e| e.to_string())?;

        self.connect_container_to_evaluation_network("tfg-server", &["tfg-server"])
            .await?;

        self.docker
            .start_container("tfg-server", None)
            .await
            .map_err(|e| e.to_string())?;

        println!("Server container started");

        Ok(())

    }

    async fn start_clients(&self, configs: &[ClientConfig]) -> Result<(), String> {

        for config in configs {

            for client_id in 0..config.num_connections {

                self.start_client_container(client_id, config).await?;

            }

        }

        Ok(())
    }



    async fn start_client_container(&self, client_id: u32, config: &ClientConfig) -> Result<(), String> {

        let cipher_suites = config.cipher_suites.join(",");
        let kx_groups = config.kx_groups.join(",");

        let env = vec![
            format!("CRYPTO_MODE={}", config.key_exchange),
            format!("CIPHER_SUITES={}", cipher_suites),
            format!("KX_GROUPS={}", kx_groups),
            "SERVER_ADDR=tfg-server:8443".to_string(),
            "SERVER_NAME=localhost".to_string(),
        ];

        let container_name = format!("tfg-client-{}", client_id + 1);

        let container_config = ContainerCreateBody{
            image: Some("tfg_project-client:latest".to_string()),
            env: Some(env),
            ..Default::default()
        };

        let options = CreateContainerOptionsBuilder::default()
            .name(&container_name)
            .build();

        self.docker
            .create_container(Some(options), container_config)
            .await
            .map_err(|e| e.to_string())?;

        self.connect_container_to_evaluation_network(&container_name, &[&container_name])
            .await?;

        self.docker
            .start_container(&container_name, None)
            .await
            .map_err(|e| e.to_string())?;

        println!("Client {} container started", client_id + 1);

        Ok(())

    }



    pub async fn stop(&self) -> Result<(), String> {

        let client_configs = self.get_client_configs().await?;

        // Stop and remove all client containers
        for config in &client_configs {

            for client_id in 0..config.num_connections {

                let container_name = format!("tfg-client-{}", client_id + 1);

                self.docker
                    .remove_container(
                        &container_name,
                        Some(RemoveContainerOptionsBuilder::default().force(true).build()),
                    )
                    .await
                    .map_err(|e| e.to_string())?;

            }

        }

        // Stop and remove the server container
        self.docker
            .remove_container(
                "tfg-server",
                Some(
                    RemoveContainerOptionsBuilder::default()
                        .force(true)
                        .build(),
                ),
            )
            .await
            .map_err(|e| e.to_string())?;

        let mut is_running = self.state.is_evaluation_running.lock().await;
        *is_running = false;

        let mut application_running = self.state.application_running.lock().await;
        *application_running = false;

        //Remove the evaluation network
        self.docker
            .remove_network("tfg-evaluation-network")
            .await
            .map_err(|e| e.to_string())?;

        println!("Server and client containers removed");

        Ok(())

    }



    async fn create_evaluation_network(&self) -> Result<(), String>{

        let network_config = NetworkCreateRequest{
            name: "tfg-evaluation-network".to_string(),
            ..Default::default()
        };

        self.docker
            .create_network(network_config)
            .await
            .map_err(|e| e.to_string())?;

        println!("Evaluation network created");

        Ok(())

    }



    async fn connect_container_to_evaluation_network(&self, container_name: &str, aliases: &[&str]) -> Result<(), String> {

        let connect_request = NetworkConnectRequest{
            container: container_name.to_string(),
            endpoint_config: Some(EndpointSettings {
                aliases: Some(aliases.iter().map(|alias| (*alias).to_string()).collect()),
                ..Default::default()
            }),
        };

        self.docker
            .connect_network("tfg-evaluation-network", connect_request)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())

    }

}
