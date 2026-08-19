use crate::dashboard::state::{ClientConfig, DashboardState, ServerConfig};
use bollard::Docker;
use bollard::models::{
    ContainerCreateBody, EndpointSettings, NetworkConnectRequest, NetworkCreateRequest,
    NetworkDisconnectRequest,
};
use bollard::query_parameters::{CreateContainerOptionsBuilder, RemoveContainerOptionsBuilder};
use chrono::Utc;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct Manager {
    state: Arc<DashboardState>,
    docker: Docker,
}

impl Manager {
    pub fn new(state: Arc<DashboardState>) -> Self {
        let docker = Docker::connect_with_local_defaults().expect("Cannot connect to Docker");

        Self { state, docker }
    }

    pub async fn start(&self) -> Result<(), String> {
        let mut is_running = self.state.is_evaluation_running.lock().await;

        if *is_running {
            return Err("An evaluation is already running".to_string());
        }

        *is_running = true;
        drop(is_running);

        let evaluation_started_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis() as i64;

        let server_config = self.get_server_config().await?;
        let client_configs = self.get_client_configs().await?;

        if let Err(e) = self.create_evaluation_network().await {
            let mut is_running = self.state.is_evaluation_running.lock().await;
            *is_running = false;
            return Err(e);
        }

        if let Err(e) = self.connect_application_to_evaluation_network().await {
            let _ = self.cleanup_evaluation(&client_configs).await;
            return Err(e);
        }

        if let Err(e) = self.start_server(&server_config).await {
            eprintln!("Error starting server: {}", e);

            let _ = self.cleanup_evaluation(&client_configs).await;

            return Err(e);
        }

        if let Err(e) = self
            .start_clients(&client_configs, evaluation_started_at)
            .await
        {
            eprintln!("Error starting clients: {}", e);

            let _ = self.cleanup_evaluation(&client_configs).await;

            return Err(e);
        }

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

        if configs.is_empty() {
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

        let env = vec![
            format!("CRYPTO_MODE={}", config.key_exchange),
            format!("CIPHER_SUITES={}", cipher_suites),
            format!("KX_GROUPS={}", kx_groups),
        ];

        let container_config = ContainerCreateBody {
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

    async fn start_clients(
        &self,
        configs: &[ClientConfig],
        evaluation_started_at: i64,
    ) -> Result<(), String> {
        for config in configs {
            for client_id in 0..config.num_connections {
                self.start_client_container(client_id, config, evaluation_started_at)
                    .await?;
            }
        }

        Ok(())
    }

    async fn start_client_container(
        &self,
        client_id: u32,
        config: &ClientConfig,
        evaluation_started_at: i64,
    ) -> Result<(), String> {
        let cipher_suites = config.cipher_suites.join(",");
        let kx_groups = config.kx_groups.join(",");

        let env = vec![
            format!("CRYPTO_MODE={}", config.key_exchange),
            format!("CIPHER_SUITES={}", cipher_suites),
            format!("KX_GROUPS={}", kx_groups),
            format!("EVALUATION_STARTED_AT={}", evaluation_started_at),
            "SERVER_ADDR=tfg-server:8443".to_string(),
            "SERVER_NAME=localhost".to_string(),
            "CONFIG_URL=https://tfg-application:8443/transfer/latest".to_string(),
        ];

        let container_name = format!("tfg-client-{}", client_id + 1);

        let container_config = ContainerCreateBody {
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

    async fn create_evaluation_network(&self) -> Result<(), String> {
        let network_config = NetworkCreateRequest {
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

    async fn connect_container_to_evaluation_network(
        &self,
        container_name: &str,
        aliases: &[&str],
    ) -> Result<(), String> {
        let connect_request = NetworkConnectRequest {
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

    async fn connect_application_to_evaluation_network(&self) -> Result<(), String> {
        let connect_request = NetworkConnectRequest {
            container: "tfg-application".to_string(),
            endpoint_config: Some(EndpointSettings {
                aliases: Some(vec!["tfg-application".to_string()]),
                ..Default::default()
            }),
        };

        self.docker
            .connect_network("tfg-evaluation-network", connect_request)
            .await
            .map_err(|e| e.to_string())?;

        println!("Application connected to evaluation network");

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), String> {
        let client_configs = self.get_client_configs().await.unwrap_or_default();

        self.cleanup_evaluation(&client_configs).await?;

        println!("Evaluation stopped and cleaned up");

        Ok(())
    }

    pub async fn reset_configuration(&self) {
        let mut server_config = self.state.server_config.lock().await;

        *server_config = None;

        drop(server_config);

        let mut client_configs = self.state.client_configs.lock().await;

        client_configs.clear();

        drop(client_configs);

        let mut evaluation_id = self.state.evaluation_id.lock().await;

        *evaluation_id = None;

        println!("Evaluation configuration reset");
    }

    async fn cleanup_evaluation(&self, configs: &[ClientConfig]) -> Result<(), String> {
        for config in configs {
            for client_id in 0..config.num_connections {
                let container_name = format!("tfg-client-{}", client_id + 1);

                let _ = self
                    .docker
                    .remove_container(
                        &container_name,
                        Some(RemoveContainerOptionsBuilder::default().force(true).build()),
                    )
                    .await;
            }
        }

        let _ = self
            .docker
            .remove_container(
                "tfg-server",
                Some(RemoveContainerOptionsBuilder::default().force(true).build()),
            )
            .await;

        let disconnect_request = NetworkDisconnectRequest {
            container: "tfg-application".to_string(),

            force: Some(true),
        };

        let _ = self
            .docker
            .disconnect_network("tfg-evaluation-network", disconnect_request)
            .await;

        let _ = self.docker.remove_network("tfg-evaluation-network").await;

        let mut application_running = self.state.application_running.lock().await;

        *application_running = false;

        drop(application_running);

        let mut is_running = self.state.is_evaluation_running.lock().await;

        *is_running = false;

        drop(is_running);

        let mut evaluation_id = self.state.evaluation_id.lock().await;

        *evaluation_id = None;

        println!("Evaluation environment cleaned up");

        Ok(())
    }
}
