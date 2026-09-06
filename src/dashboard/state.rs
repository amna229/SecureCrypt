//! Shared state and data structures used by the dashboard.
//!
//! This module defines the configuration structures, dashboard events,
//! and shared runtime state used by the dashboard services and handlers.

use crate::crypto::crypto_mode::CryptoMode;
use crate::dashboard::services::application::ApplicationService;

use std::collections::HashSet;
use tokio::process::Child;
use tokio::sync::{Mutex, broadcast};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerConfig {
    pub key_exchange: CryptoMode,
    pub cipher_suites: Vec<String>,
    pub kx_groups: Vec<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ClientConfig {
    pub key_exchange: CryptoMode,
    pub cipher_suites: Vec<String>,
    pub kx_groups: Vec<String>,
    pub server_addr: String,
    pub server_name: String,
    pub num_connections: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EvaluationEvent {
    pub event_type: String,
    pub evaluation_id: Uuid,
    pub client_id: Option<i32>,
    pub completed: i64,
    pub total: i64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum DashboardEvent {
    #[serde(rename = "transfer")]
    Transfer {
        event_type: String,
        evaluation_id: Uuid,
        execution_id: String,
        client_id: i32,
        operation: String,
        success: bool,
        completed: i64,
        total: i64,
        all_completed: bool,
    },

    #[serde(rename = "evaluation")]
    Evaluation {
        event_type: String,
        evaluation_id: Uuid,
        client_id: Option<i32>,
        completed: i64,
        total: i64,
    },
}

pub struct DashboardState {
    pub client_configs: Mutex<Vec<ClientConfig>>,
    pub server_config: Mutex<Option<ServerConfig>>,
    pub is_evaluation_running: Mutex<bool>,
    pub application_running: Mutex<bool>,
    pub evaluation_id: Mutex<Option<Uuid>>,
    pub db: crate::dashboard::database::storage::DashboardStorage,
    pub resource_monitor:
        Mutex<Option<crate::dashboard::services::resource_monitor::ResourceMonitor>>,
    pub resource_monitor_token: Mutex<CancellationToken>,
    pub transfer_events: broadcast::Sender<DashboardEvent>,
    pub evaluation_events: broadcast::Sender<EvaluationEvent>,
    pub application_service: Mutex<Option<ApplicationService>>,
    pub application_process: Mutex<Option<Child>>,
    pub application_program: Mutex<Option<String>>,
    pub completed_clients: Mutex<HashSet<(Uuid, i32)>>,
}

impl DashboardState {
    pub fn new(db: crate::dashboard::database::storage::DashboardStorage) -> Self {
        let (transfer_events, _) = broadcast::channel(100);
        let (evaluation_events, _) = broadcast::channel(100);

        Self {
            client_configs: Mutex::new(Vec::new()),
            server_config: Mutex::new(None),
            is_evaluation_running: Mutex::new(false),
            application_running: Mutex::new(false),
            evaluation_id: Mutex::new(None),
            db,
            resource_monitor: Mutex::new(None),
            resource_monitor_token: Mutex::new(CancellationToken::new()),
            transfer_events,
            evaluation_events,
            application_service: Mutex::new(None),
            application_process: Mutex::new(None),
            application_program: Mutex::new(None),
            completed_clients: Mutex::new(HashSet::new()),
        }
    }

    pub async fn is_evaluation_running(&self) -> bool {
        *self.is_evaluation_running.lock().await
    }

    pub async fn set_application_program(&self, application_program: String) {
        let mut program = self.application_program.lock().await;
        *program = Some(application_program);
    }

    pub async fn application_program(&self) -> Option<String> {
        self.application_program.lock().await.clone()
    }
}
