//! Dashboard notification handling.
//!
//! This module sends transfer lifecycle notifications
//! from the application service to the dashboard.

use reqwest::Client;
use uuid::Uuid;

const DASHBOARD_URL: &str = "http://dashboard:3000";

/// Notification sent to the dashboard when a transfer changes state.
#[derive(Debug, serde::Serialize)]
pub(crate) struct TransferNotification {
    pub event_type: String,
    pub evaluation_id: Uuid,
    pub execution_id: String,
    pub client_id: i32,
    pub operation: String,
    pub success: bool,
}

/// Notifies the dashboard about a transfer lifecycle event.
///
/// Notification failures are logged but do not interrupt
/// the main application operation.
pub(crate) async fn notify_dashboard(
    event_type: &str,
    evaluation_id: Uuid,
    execution_id: &str,
    client_id: i32,
    operation: &str,
    success: bool,
) {
    let notification = TransferNotification {
        event_type: event_type.to_string(),
        evaluation_id,
        execution_id: execution_id.to_string(),
        client_id,
        operation: operation.to_string(),
        success,
    };

    let client = Client::new();

    let result = client
        .post(format!("{}/eval/transfer-event", DASHBOARD_URL))
        .json(&notification)
        .send()
        .await;

    if let Err(error) = result {
        eprintln!(
            "Error notifying dashboard about transfer {}: {}",
            execution_id, error
        );
    }
}
