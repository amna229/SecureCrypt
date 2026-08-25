//! Configuration handlers.
//!
//! This module contains the HTTP handlers responsible for storing
//! server and client configurations and checking their status.

use axum::{Json, extract::State};

use std::sync::Arc;

use crate::dashboard::state::{ClientConfig, DashboardState, ServerConfig};

/// Stores the server configuration used for an evaluation.
pub async fn save_server_config(
    State(state): State<Arc<DashboardState>>,
    Json(config): Json<ServerConfig>,
) {
    if state.is_evaluation_running().await {
        println!("Cannot modify server configuration while an evaluation is running");

        return;
    }

    println!("{:#?}", config);

    let mut server_config = state.server_config.lock().await;

    *server_config = Some(config);

    println!("Server configuration saved");
}

/// Stores the client configuration used for an evaluation.
pub async fn save_client_config(
    State(state): State<Arc<DashboardState>>,
    Json(config): Json<ClientConfig>,
) {
    if state.is_evaluation_running().await {
        println!("Cannot modify client configuration while an evaluation is running");

        return;
    }

    println!("{:#?}", config);

    let mut client_configs = state.client_configs.lock().await;

    client_configs.clear();
    client_configs.push(config);

    println!(
        "Client configuration saved. Total configurations: {}",
        client_configs.len()
    );
}

/// Returns whether a client configuration is available.
pub async fn client_status(State(state): State<Arc<DashboardState>>) -> Json<bool> {
    let clients = state.client_configs.lock().await;

    Json(!clients.is_empty())
}

/// Returns whether the application is currently running.
pub async fn application_status(State(state): State<Arc<DashboardState>>) -> Json<bool> {
    let running = state.application_running.lock().await;

    Json(*running)
}
