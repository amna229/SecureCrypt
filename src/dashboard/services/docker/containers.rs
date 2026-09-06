//! Docker container management.

use crate::dashboard::services::manager::Manager;
use crate::dashboard::state::{ClientConfig, ServerConfig};

use bollard::models::ContainerCreateBody;
use bollard::query_parameters::{CreateContainerOptionsBuilder, RemoveContainerOptionsBuilder};

impl Manager {
    /// Creates and starts the evaluation server container.
    pub(crate) async fn start_server_container(&self, config: &ServerConfig) -> Result<(), String> {
        let docker = self
            .docker
            .as_ref()
            .ok_or_else(|| "Docker is not available".to_string())?;

        let cipher_suites = config.cipher_suites.join(",");

        let kx_groups = config.kx_groups.join(",");

        let env = vec![
            format!("CRYPTO_MODE={}", config.key_exchange),
            format!("CIPHER_SUITES={}", cipher_suites),
            format!("KX_GROUPS={}", kx_groups),
        ];

        let container_config = ContainerCreateBody {
            image: Some("securecrypt-server:latest".to_string()),
            env: Some(env),
            ..Default::default()
        };

        let options = CreateContainerOptionsBuilder::default()
            .name("securecrypt-server")
            .build();

        docker
            .create_container(Some(options), container_config)
            .await
            .map_err(|e| e.to_string())?;

        self.connect_container_to_evaluation_network("securecrypt-server", &["securecrypt-server"])
            .await?;

        docker
            .start_container("securecrypt-server", None)
            .await
            .map_err(|e| e.to_string())?;

        println!("Server container started");

        Ok(())
    }

    /// Creates and starts all configured client containers.
    pub(crate) async fn start_clients(
        &self,
        configs: &[ClientConfig],
        evaluation_started_at: i64,
    ) -> Result<(), String> {
        if self.docker.is_none() {
            return Err("Docker is not available".to_string());
        }

        for config in configs {
            for client_id in 0..config.num_connections {
                self.start_client_container(client_id, config, evaluation_started_at)
                    .await?;
            }
        }

        Ok(())
    }

    /// Creates and starts a single client container.
    pub(crate) async fn start_client_container(
        &self,
        client_id: u32,
        config: &ClientConfig,
        evaluation_started_at: i64,
    ) -> Result<(), String> {
        let docker = self
            .docker
            .as_ref()
            .ok_or_else(|| "Docker is not available".to_string())?;

        let cipher_suites = config.cipher_suites.join(",");

        let kx_groups = config.kx_groups.join(",");

        let evaluation_id = {
            let evaluation_id = self.state.evaluation_id.lock().await;

            evaluation_id.ok_or_else(|| "No active evaluation ID".to_string())?
        };

        let actual_client_id = client_id + 1;

        let env = vec![
            format!("CRYPTO_MODE={}", config.key_exchange),
            format!("CIPHER_SUITES={}", cipher_suites),
            format!("KX_GROUPS={}", kx_groups),
            format!("EVALUATION_ID={}", evaluation_id),
            format!("CLIENT_ID={}", actual_client_id),
            format!("EVALUATION_STARTED_AT={}", evaluation_started_at),
            format!("SERVER_ADDR={}", config.server_addr),
            format!("SERVER_NAME={}", config.server_name),
            "CONFIG_URL=https://securecrypt-application:8443/transfer/latest".to_string(),
        ];

        let container_name = format!("securecrypt-client-{}", actual_client_id);

        let container_config = ContainerCreateBody {
            image: Some("securecrypt-client:latest".to_string()),
            env: Some(env),
            ..Default::default()
        };

        let options = CreateContainerOptionsBuilder::default()
            .name(&container_name)
            .build();

        docker
            .create_container(Some(options), container_config)
            .await
            .map_err(|e| e.to_string())?;

        self.connect_container_to_evaluation_network(&container_name, &[&container_name])
            .await?;

        docker
            .start_container(&container_name, None)
            .await
            .map_err(|e| e.to_string())?;

        let started_clients = client_id + 1;

        println!("Client {} container started", actual_client_id);

        self.send_evaluation_event(
            "client-started",
            evaluation_id,
            Some(actual_client_id as i32),
            started_clients as i64,
            config.num_connections as i64,
        );

        Ok(())
    }

    /// Removes a client container.
    pub(crate) async fn remove_client_container(&self, container_name: &str) {
        let Some(docker) = self.docker.as_ref() else {
            return;
        };

        let _ = docker
            .remove_container(
                container_name,
                Some(RemoveContainerOptionsBuilder::default().force(true).build()),
            )
            .await;
    }

    /// Removes the evaluation server container.
    pub(crate) async fn remove_server_container(&self) {
        let Some(docker) = self.docker.as_ref() else {
            return;
        };

        let _ = docker
            .remove_container(
                "securecrypt-server",
                Some(RemoveContainerOptionsBuilder::default().force(true).build()),
            )
            .await;
    }
}
