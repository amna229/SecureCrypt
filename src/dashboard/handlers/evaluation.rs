//! Evaluation handlers.
//!
//! This module contains the HTTP handlers responsible for starting,
//! stopping, resetting, and querying the current evaluation.

use axum::{Json, extract::State, http::StatusCode};

use std::sync::Arc;

use uuid::Uuid;

use crate::dashboard::{services::manager::Manager, state::DashboardState};

#[derive(serde::Serialize)]
pub struct CurrentEvaluationResponse {
    pub evaluation_id: Option<Uuid>,
}

/// Starts the evaluation environment.
pub async fn start_evaluation(State(state): State<Arc<DashboardState>>) {
    println!("Starting evaluation environment...");

    let manager = Manager::new(state);

    match manager.start().await {
        Ok(()) => {
            println!("Evaluation environment started successfully");
        }

        Err(error) => {
            eprintln!("Cannot start evaluation environment: {}", error);
        }
    }
}

/// Stops the evaluation environment.
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
