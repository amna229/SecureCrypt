use crate::dashboard::metrics::repository::EvaluationRepository;
use crate::dashboard::services::manager::Manager;
use crate::dashboard::services::resource_monitor::{MonitoredContainer, ResourceMonitor};
use crate::dashboard::state::ServerConfig;

use chrono::Utc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

impl Manager {
    /// Starts a complete evaluation environment.
    ///
    /// The lifecycle includes creating the evaluation record,
    /// preparing the Docker environment, starting the server and
    /// clients, and starting resource monitoring.
    pub async fn start(&self) -> Result<(), String> {
        let mut is_running = self.state.is_evaluation_running.lock().await;

        if *is_running {
            return Err("An evaluation is already running".to_string());
        }

        *is_running = true;

        drop(is_running);

        let resource_monitor_token = {
            let mut token = self.state.resource_monitor_token.lock().await;

            *token = CancellationToken::new();

            token.clone()
        };

        let evaluation_started_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_millis() as i64;

        let server_config = self.get_server_config().await?;

        let client_configs = self.get_client_configs().await?;

        let evaluation_id = Uuid::new_v4();
        let started_at = Utc::now();

        let num_clients: i32 = client_configs
            .iter()
            .map(|config| config.num_connections as i32)
            .sum();

        let evaluation_repository = EvaluationRepository::new(self.state.db.clone());

        evaluation_repository
            .create(
                evaluation_id,
                &server_config.key_exchange.to_string(),
                num_clients,
                started_at,
            )
            .await
            .map_err(|e| format!("Error creating evaluation: {}", e))?;

        {
            let mut current_evaluation_id = self.state.evaluation_id.lock().await;

            *current_evaluation_id = Some(evaluation_id);
        }

        self.send_evaluation_event(
            "environment-starting",
            evaluation_id,
            None,
            0,
            num_clients as i64,
        );

        if let Err(e) = self.create_evaluation_network().await {
            let _ = self.cleanup_evaluation(&client_configs).await;

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

        let mut monitored_containers = vec![
            MonitoredContainer {
                name: "securecrypt-application".to_string(),
                role: "application".to_string(),
                client_id: None,
            },
            MonitoredContainer {
                name: "securecrypt-server".to_string(),
                role: "server".to_string(),
                client_id: None,
            },
        ];

        for config in &client_configs {
            for client_id in 1..=config.num_connections {
                monitored_containers.push(MonitoredContainer {
                    name: format!("securecrypt-client-{}", client_id),
                    role: "client".to_string(),
                    client_id: Some(client_id as i32),
                });
            }
        }

        let resource_monitor = ResourceMonitor::start(
            self.docker.clone(),
            self.state.db.clone(),
            evaluation_id,
            monitored_containers,
            resource_monitor_token,
        );

        {
            let mut monitor = self.state.resource_monitor.lock().await;

            *monitor = Some(resource_monitor);
        }

        self.send_evaluation_event(
            "environment-ready",
            evaluation_id,
            None,
            num_clients as i64,
            num_clients as i64,
        );

        println!(
            "Server, clients and resource monitoring \
             started successfully"
        );

        Ok(())
    }

    /// Starts the configured server and marks the application
    /// as running.
    async fn start_server(&self, config: &ServerConfig) -> Result<(), String> {
        println!("Starting server...");

        self.start_server_container(config).await?;

        let mut application_running = self.state.application_running.lock().await;

        *application_running = true;

        let evaluation_id = {
            let evaluation_id = self.state.evaluation_id.lock().await;

            evaluation_id.ok_or_else(|| "No active evaluation ID".to_string())?
        };

        self.send_evaluation_event("server-started", evaluation_id, None, 0, 0);

        Ok(())
    }

    /// Stops the current evaluation and releases
    /// its associated resources.
    pub async fn stop(&self) -> Result<(), String> {
        let client_configs = self.get_client_configs().await.unwrap_or_default();

        let evaluation_id = {
            let evaluation_id = self.state.evaluation_id.lock().await;

            *evaluation_id
        };

        if let Some(evaluation_id) = evaluation_id {
            let total_clients = client_configs
                .iter()
                .map(|config| config.num_connections as i32)
                .sum::<i32>();

            self.send_evaluation_event(
                "environment-stopping",
                evaluation_id,
                None,
                0,
                total_clients as i64,
            );
        }

        {
            let mut monitor = self.state.resource_monitor.lock().await;

            if let Some(resource_monitor) = monitor.take() {
                resource_monitor.stop().await;
            }
        }

        if let Some(evaluation_id) = evaluation_id {
            let repository = EvaluationRepository::new(self.state.db.clone());

            repository
                .finish(evaluation_id, "completed")
                .await
                .map_err(|e| format!("Error finishing evaluation: {}", e))?;
        }

        self.cleanup_evaluation(&client_configs).await?;

        println!("Evaluation stopped and cleaned up");

        Ok(())
    }
}
