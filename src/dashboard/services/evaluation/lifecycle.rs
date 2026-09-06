//! Evaluation lifecycle management.
//!
//! This module coordinates the creation, execution and termination
//! of an evaluation.
//!
//! The external application executable is received by `start()` and
//! is never hardcoded in SecureCrypt.

use crate::dashboard::database::storage::DashboardStorage;
use crate::dashboard::metrics::repository::EvaluationRepository;
use crate::dashboard::services::application::ApplicationService;
use crate::dashboard::services::manager::Manager;
use crate::dashboard::services::resource_monitor::{MonitoredContainer, ResourceMonitor};
use crate::dashboard::state::{ClientConfig, ServerConfig};

use chrono::Utc;
use reqwest::Client;
use serde_json::json;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpStream;
use tokio::time::{Duration, sleep};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

impl Manager {
    /// Starts a complete evaluation environment.
    ///
    /// The executable used by the external application is supplied
    /// by the caller, keeping the evaluation system independent of
    /// any concrete application.
    pub async fn start(&self, application_program: &str) -> Result<(), String> {
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
            .map_err(|error| error.to_string())?
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
            .map_err(|error| format!("Error creating evaluation: {}", error))?;

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

        if let Err(error) = self
            .start_external_application(
                application_program,
                &server_config,
                &client_configs,
                evaluation_started_at,
            )
            .await
        {
            eprintln!("Error starting external application: {}", error);

            let _ = self.cleanup_evaluation(&client_configs).await;

            let mut running = self.state.is_evaluation_running.lock().await;

            *running = false;

            return Err(error);
        }

        if !matches!(&self.state.db, DashboardStorage::Memory(_)) {
            let monitored_containers = self.build_monitored_containers(&client_configs);

            let resource_monitor = ResourceMonitor::start(
                self.docker
                    .clone()
                    .ok_or_else(|| "Docker is unavailable".to_string())?,
                self.state.db.clone(),
                evaluation_id,
                monitored_containers,
                resource_monitor_token,
            );

            let mut monitor = self.state.resource_monitor.lock().await;

            *monitor = Some(resource_monitor);
        } else {
            println!("Standalone mode: Docker resource monitoring disabled");
        }

        self.send_evaluation_event(
            "environment-ready",
            evaluation_id,
            None,
            num_clients as i64,
            num_clients as i64,
        );

        println!(
            "External application and resource monitoring \
             started successfully"
        );

        Ok(())
    }

    /// Starts the external application as the evaluation server
    /// and as the configured evaluation clients.
    ///
    /// Each standalone process receives its own control endpoint.
    async fn start_external_application(
        &self,
        application_program: &str,
        server_config: &ServerConfig,
        client_configs: &[ClientConfig],
        evaluation_started_at: i64,
    ) -> Result<(), String> {
        let service = ApplicationService::new();

        let evaluation_id = self.current_evaluation_id_string().await?;

        let standalone = matches!(&self.state.db, DashboardStorage::Memory(_));

        let mut server_environment = HashMap::new();

        server_environment.insert("ROLE".to_string(), "server".to_string());

        server_environment.insert(
            "CRYPTO_MODE".to_string(),
            server_config.key_exchange.to_string(),
        );

        server_environment.insert(
            "CIPHER_SUITES".to_string(),
            server_config.cipher_suites.join(","),
        );

        server_environment.insert("KX_GROUPS".to_string(), server_config.kx_groups.join(","));

        server_environment.insert(
            "SECURECRYPT_CONTROL_ADDR".to_string(),
            "127.0.0.1:9090".to_string(),
        );

        println!(
            "Starting external application server using '{}'",
            application_program
        );

        service
            .start(application_program, &[], &server_environment)
            .await?;

        let server_configuration = json!({
            "evaluation_id":
                evaluation_id.clone(),

            "crypto_mode":
                server_config
                    .key_exchange
                    .to_string(),

            "cipher_suites":
                server_config
                    .cipher_suites
                    .clone(),

            "kx_groups":
                server_config
                    .kx_groups
                    .clone(),

            "server_addr":
                null,

            "server_name":
                null,

            "listen_addr":
                "0.0.0.0:8443"
        });

        Self::send_application_configuration("127.0.0.1:9090", server_configuration).await?;

        let mut server_ready = false;

        for _ in 0..50 {
            if TcpStream::connect("127.0.0.1:8443").await.is_ok() {
                server_ready = true;

                println!(
                    "External application server is ready \
                     on 127.0.0.1:8443"
                );

                break;
            }

            sleep(Duration::from_millis(100)).await;
        }

        if !server_ready {
            return Err("External application server did not become \
                 ready on 127.0.0.1:8443"
                .to_string());
        }

        let mut global_client_id = 1u32;

        for config in client_configs {
            for _ in 0..config.num_connections {
                let control_port = 9090 + global_client_id;

                let control_addr = format!("127.0.0.1:{}", control_port);

                let mut client_environment = HashMap::new();

                client_environment.insert("ROLE".to_string(), "client".to_string());

                client_environment
                    .insert("CRYPTO_MODE".to_string(), config.key_exchange.to_string());

                client_environment
                    .insert("CIPHER_SUITES".to_string(), config.cipher_suites.join(","));

                client_environment.insert("KX_GROUPS".to_string(), config.kx_groups.join(","));

                client_environment.insert("CLIENT_ID".to_string(), global_client_id.to_string());

                client_environment.insert(
                    "EVALUATION_STARTED_AT".to_string(),
                    evaluation_started_at.to_string(),
                );

                client_environment.insert("EVALUATION_ID".to_string(), evaluation_id.clone());

                client_environment
                    .insert("SECURECRYPT_CONTROL_ADDR".to_string(), control_addr.clone());

                /*
                 * In standalone mode the default Docker
                 * hostname is not resolvable.
                 *
                 * When the user keeps the default Docker
                 * address, translate it to localhost.
                 *
                 * Any explicitly configured address is used
                 * unchanged. This allows the client to connect
                 * to another host or port.
                 */
                let server_addr = if standalone && config.server_addr == "securecrypt-server:8443" {
                    "127.0.0.1:8443".to_string()
                } else {
                    config.server_addr.clone()
                };

                let server_name = config.server_name.clone();

                println!(
                    "Starting external application client {} \
                     using '{}'",
                    global_client_id, application_program
                );

                service
                    .start(application_program, &[], &client_environment)
                    .await?;

                let client_configuration = json!({
                    "evaluation_id":
                        evaluation_id.clone(),

                    "crypto_mode":
                        config
                            .key_exchange
                            .to_string(),

                    "cipher_suites":
                        config
                            .cipher_suites
                            .clone(),

                    "kx_groups":
                        config
                            .kx_groups
                            .clone(),

                    "server_addr":
                        server_addr,

                    "server_name":
                        server_name,

                    "listen_addr":
                        null
                });

                Self::send_application_configuration(&control_addr, client_configuration).await?;

                self.send_evaluation_event(
                    "client-started",
                    Uuid::parse_str(&evaluation_id)
                        .map_err(|error| format!("Invalid evaluation ID: {}", error))?,
                    Some(global_client_id as i32),
                    global_client_id as i64,
                    client_configs
                        .iter()
                        .map(|item| item.num_connections)
                        .sum::<u32>() as i64,
                );

                global_client_id += 1;
            }
        }

        {
            let mut application_service_state = self.state.application_service.lock().await;

            *application_service_state = Some(service);
        }

        let evaluation_id = {
            let evaluation_id = self.state.evaluation_id.lock().await;

            evaluation_id.ok_or_else(|| "No active evaluation ID".to_string())?
        };

        {
            let mut application_running = self.state.application_running.lock().await;

            *application_running = true;
        }

        self.send_evaluation_event("server-started", evaluation_id, None, 0, 0);

        Ok(())
    }

    /// Sends an evaluation configuration to an external application.
    async fn send_application_configuration(
        control_addr: &str,
        config: serde_json::Value,
    ) -> Result<(), String> {
        let client = Client::new();

        let url = format!("http://{}/configure", control_addr);

        for _ in 0..50 {
            match client.post(&url).json(&config).send().await {
                Ok(response) if response.status().is_success() => {
                    println!(
                        "Application configured successfully \
                         at {}",
                        control_addr
                    );

                    return Ok(());
                }

                _ => {
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }

        Err(format!(
            "Could not configure external application \
             at {}",
            control_addr
        ))
    }

    /// Builds the list of Docker containers monitored during
    /// an evaluation.
    fn build_monitored_containers(
        &self,
        client_configs: &[ClientConfig],
    ) -> Vec<MonitoredContainer> {
        let mut monitored_containers = vec![MonitoredContainer {
            name: "securecrypt-server".to_string(),
            role: "server".to_string(),
            client_id: None,
        }];

        for config in client_configs {
            for client_id in 1..=config.num_connections {
                monitored_containers.push(MonitoredContainer {
                    name: format!("securecrypt-client-{}", client_id),
                    role: "client".to_string(),
                    client_id: Some(client_id as i32),
                });
            }
        }

        monitored_containers
    }

    /// Returns the current evaluation identifier as a string.
    async fn current_evaluation_id_string(&self) -> Result<String, String> {
        let evaluation_id = self.state.evaluation_id.lock().await;

        evaluation_id
            .map(|id| id.to_string())
            .ok_or_else(|| "No active evaluation ID".to_string())
    }

    /// Stops the current evaluation and releases all resources.
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
            let mut application_service_state = self.state.application_service.lock().await;

            if let Some(application_service) = application_service_state.take() {
                application_service.stop().await?;
            }
        }

        {
            let mut application_running = self.state.application_running.lock().await;

            *application_running = false;
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
                .map_err(|error| format!("Error finishing evaluation: {}", error))?;
        }

        self.cleanup_evaluation(&client_configs).await?;

        {
            let mut running = self.state.is_evaluation_running.lock().await;

            *running = false;
        }

        println!("Evaluation stopped and cleaned up");

        Ok(())
    }
}
