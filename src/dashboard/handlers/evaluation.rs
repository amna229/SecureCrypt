//! Evaluation HTTP handlers.
//!
//! This module exposes the HTTP operations used by the dashboard
//! to start, stop, reset and inspect an evaluation.
//!
//! The executable of the external application is obtained from
//! the shared dashboard state.

use axum::{Json, extract::State, http::StatusCode};

use std::sync::Arc;

use uuid::Uuid;

use crate::dashboard::{services::manager::Manager, state::DashboardState};

#[derive(serde::Serialize)]
pub struct CurrentEvaluationResponse {
    pub evaluation_id: Option<Uuid>,
}

/// Starts the evaluation using the external application executable
/// stored in the dashboard state.
pub async fn start_evaluation(
    State(state): State<Arc<DashboardState>>,
) -> Result<StatusCode, String> {
    let application_program = state
        .application_program()
        .await
        .ok_or_else(|| "External application executable is not configured".to_string())?;

    println!(
        "Starting evaluation environment using '{}'",
        application_program
    );

    let manager = Manager::new(state);

    manager.start(&application_program).await?;

    println!("Evaluation environment started successfully");

    Ok(StatusCode::OK)
}

/// Stops the current evaluation environment.
pub async fn stop_application(State(state): State<Arc<DashboardState>>) -> Result<(), String> {
    let manager = Manager::new(state);

    manager.stop().await?;

    Ok(())
}

/// Resets the evaluation configuration.
pub async fn reset_evaluation(State(state): State<Arc<DashboardState>>) -> StatusCode {
    let manager = Manager::new(state);

    manager.reset_configuration().await;

    StatusCode::OK
}

/// Returns the identifier of the current evaluation.
pub async fn current_evaluation(
    State(state): State<Arc<DashboardState>>,
) -> Json<CurrentEvaluationResponse> {
    let evaluation_id = state.evaluation_id.lock().await;

    Json(CurrentEvaluationResponse {
        evaluation_id: *evaluation_id,
    })
}
