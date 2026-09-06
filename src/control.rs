//! Evaluation control endpoint.
//!
//! This module provides the HTTP endpoint used to receive the
//! configuration of an evaluation from the SecureCrypt dashboard.

use axum::{Json, Router, extract::State, routing::post};

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationConfig {
    pub evaluation_id: String,
    pub crypto_mode: String,
    pub cipher_suites: Vec<String>,
    pub kx_groups: Vec<String>,
    pub server_addr: Option<String>,
    pub server_name: Option<String>,
    pub listen_addr: Option<String>,
}

struct ControlState {
    sender: std::sync::Mutex<Option<oneshot::Sender<EvaluationConfig>>>,
}

async fn configure(
    State(state): State<Arc<ControlState>>,
    Json(config): Json<EvaluationConfig>,
) -> &'static str {
    if let Some(sender) = state.sender.lock().unwrap().take() {
        let _ = sender.send(config);
        "Configuration received"
    } else {
        "Configuration already received"
    }
}

pub async fn wait_for_configuration(
    addr: &str,
) -> Result<EvaluationConfig, Box<dyn std::error::Error + Send + Sync>> {
    let (sender, receiver) = oneshot::channel();

    let state = Arc::new(ControlState {
        sender: std::sync::Mutex::new(Some(sender)),
    });

    let app = Router::new()
        .route("/configure", post(configure))
        .with_state(state);

    let listener = TcpListener::bind(addr).await?;

    println!("SecureCrypt control endpoint listening on {}", addr);

    tokio::spawn(async move {
        if let Err(error) = axum::serve(listener, app).await {
            eprintln!("Control endpoint error: {}", error);
        }
    });

    Ok(receiver.await?)
}

/// Starts the control endpoint using the address
/// supplied through `SECURECRYPT_CONTROL_ADDR`.
///
/// When the variable is not defined, port 9090 is used
/// for backwards compatibility.
pub async fn wait_for_configuration_from_environment()
-> Result<EvaluationConfig, Box<dyn std::error::Error + Send + Sync>> {
    let addr =
        std::env::var("SECURECRYPT_CONTROL_ADDR").unwrap_or_else(|_| "0.0.0.0:9090".to_string());

    wait_for_configuration(&addr).await
}
