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





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    fn create_test_pool() -> PgPool {
        PgPoolOptions::new()
            .connect_lazy("postgres://test:test@localhost/test")
            .expect("A lazy PostgreSQL pool should be created")
    }

    #[tokio::test]
    async fn dashboard_state_is_initialized_correctly() {
        let pool = create_test_pool();
        let state = DashboardState::new(pool);

        assert!(!state.is_evaluation_running().await);

        assert!(state.client_configs.lock().await.is_empty());
        assert!(state.server_config.lock().await.is_none());
        assert!(state.evaluation_id.lock().await.is_none());
        assert!(!*state.application_running.lock().await);
    }

    #[tokio::test]
    async fn dashboard_state_accepts_server_configuration() {
        let pool = create_test_pool();
        let state = DashboardState::new(pool);

        let config = ServerConfig {
            key_exchange: crate::crypto::crypto_mode::CryptoMode::new(
                "classical".to_string(),
            ),
            cipher_suites: vec![
                "TLS13_AES_128_GCM_SHA256".to_string(),
            ],
            kx_groups: vec!["X25519".to_string()],
        };

        *state.server_config.lock().await = Some(config);

        let stored_config = state
            .server_config
            .lock()
            .await
            .clone()
            .expect("The server configuration should be stored");

        assert_eq!(stored_config.key_exchange.to_string(), "classical");
        assert_eq!(stored_config.cipher_suites.len(), 1);
        assert_eq!(stored_config.kx_groups.len(), 1);
    }

    #[tokio::test]
    async fn dashboard_state_accepts_client_configurations() {
        let pool = create_test_pool();
        let state = DashboardState::new(pool);

        let config = ClientConfig {
            key_exchange: crate::crypto::crypto_mode::CryptoMode::new(
                "post_quantum".to_string(),
            ),
            cipher_suites: vec![
                "TLS13_AES_128_GCM_SHA256".to_string(),
            ],
            kx_groups: vec!["MLKEM768".to_string()],
            num_connections: 3,
        };

        state.client_configs.lock().await.push(config);

        let configs = state.client_configs.lock().await;

        assert_eq!(configs.len(), 1);
        assert_eq!(
            configs[0].key_exchange.to_string(),
            "post_quantum"
        );
        assert_eq!(configs[0].num_connections, 3);
    }

    #[test]
    fn evaluation_event_serializes_correctly() {
        let evaluation_id = Uuid::new_v4();

        let event = EvaluationEvent {
            event_type: "evaluation-started".to_string(),
            evaluation_id,
            client_id: None,
            completed: 0,
            total: 3,
        };

        let value = serde_json::to_value(&event)
            .expect("Evaluation event should serialize successfully");

        assert_eq!(value["event_type"], "evaluation-started");
        assert_eq!(value["evaluation_id"], evaluation_id.to_string());
        assert!(value["client_id"].is_null());
        assert_eq!(value["completed"], 0);
        assert_eq!(value["total"], 3);
    }

    #[test]
    fn transfer_dashboard_event_serializes_correctly() {
        let evaluation_id = Uuid::new_v4();

        let event = DashboardEvent::Transfer {
            event_type: "transfer-completed".to_string(),
            evaluation_id,
            execution_id: "execution-01".to_string(),
            client_id: 1,
            operation: "upload".to_string(),
            success: true,
            completed: 1,
            total: 2,
            all_completed: false,
        };

        let value = serde_json::to_value(&event)
            .expect("Transfer dashboard event should serialize successfully");

        assert_eq!(value["type"], "transfer");
        assert_eq!(value["event_type"], "transfer-completed");
        assert_eq!(value["execution_id"], "execution-01");
        assert_eq!(value["client_id"], 1);
        assert_eq!(value["operation"], "upload");
        assert_eq!(value["success"], true);
        assert_eq!(value["completed"], 1);
        assert_eq!(value["total"], 2);
        assert_eq!(value["all_completed"], false);
    }

    #[test]
    fn evaluation_dashboard_event_serializes_correctly() {
        let evaluation_id = Uuid::new_v4();

        let event = DashboardEvent::Evaluation {
            event_type: "evaluation-completed".to_string(),
            evaluation_id,
            client_id: Some(1),
            completed: 3,
            total: 3,
        };

        let value = serde_json::to_value(&event)
            .expect("Evaluation dashboard event should serialize successfully");

        assert_eq!(value["type"], "evaluation");
        assert_eq!(value["event_type"], "evaluation-completed");
        assert_eq!(value["client_id"], 1);
        assert_eq!(value["completed"], 3);
        assert_eq!(value["total"], 3);
    }
}