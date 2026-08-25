//! Shared state and data structures used by the dashboard.
//!
//! This module defines the configuration structures, dashboard events,
//! and shared runtime state used by the dashboard services and handlers.

use sqlx::PgPool;
use tokio::sync::{Mutex, broadcast};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// Configuration used to start the evaluation server.
///
/// The configuration specifies the cryptographic mode, cipher suites,
/// and key exchange groups used by the server.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerConfig {
    pub key_exchange: crate::crypto::crypto_mode::CryptoMode,
    pub cipher_suites: Vec<String>,
    pub kx_groups: Vec<String>,
}

/// Configuration used to start evaluation clients.
///
/// In addition to the cryptographic configuration, this structure
/// specifies the number of client connections to create.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ClientConfig {
    pub key_exchange: crate::crypto::crypto_mode::CryptoMode,
    pub cipher_suites: Vec<String>,
    pub kx_groups: Vec<String>,
    pub server_addr: String,
    pub server_name: String,
    pub num_connections: u32,
}

/// Represents an event related to the lifecycle and progress of
/// an evaluation environment.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EvaluationEvent {
    pub event_type: String,
    pub evaluation_id: Uuid,
    pub client_id: Option<i32>,
    pub completed: i64,
    pub total: i64,
}

/// Represents an event sent by the dashboard to notify clients
/// about evaluation or transfer progress.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum DashboardEvent {
    /// Represents a transfer-related event.
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

    /// Represents an evaluation lifecycle event.
    #[serde(rename = "evaluation")]
    Evaluation {
        event_type: String,
        evaluation_id: Uuid,
        client_id: Option<i32>,
        completed: i64,
        total: i64,
    },
}

/// Shared runtime state used by the dashboard.
///
/// The state contains the current evaluation configuration, runtime
/// status, database connection pool, resource monitor, and event
/// channels shared by the dashboard handlers and services.
pub struct DashboardState {
    pub client_configs: Mutex<Vec<ClientConfig>>,
    pub server_config: Mutex<Option<ServerConfig>>,
    pub is_evaluation_running: Mutex<bool>,
    pub application_running: Mutex<bool>,
    pub evaluation_id: Mutex<Option<Uuid>>,
    pub db: PgPool,
    pub resource_monitor:
        Mutex<Option<crate::dashboard::services::resource_monitor::ResourceMonitor>>,
    pub resource_monitor_token: Mutex<CancellationToken>,
    pub transfer_events: broadcast::Sender<DashboardEvent>,
    pub evaluation_events: broadcast::Sender<EvaluationEvent>,
}

impl DashboardState {
    /// Creates a new dashboard state with the provided database pool.
    ///
    /// All runtime values are initialized to their default inactive
    /// state and broadcast channels are created for evaluation events
    /// and transfer events.
    pub fn new(db: PgPool) -> Self {
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
        }
    }

    /// Returns whether an evaluation is currently running.
    pub async fn is_evaluation_running(&self) -> bool {
        *self.is_evaluation_running.lock().await
    }
}
