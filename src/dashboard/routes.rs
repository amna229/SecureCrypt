//! Dashboard HTTP route configuration.
//!
//! This module defines the HTTP routes exposed by the dashboard,
//! including page rendering, configuration management, evaluation
//! control, event streaming, and result retrieval.

use crate::dashboard::{
    handlers::{
        configuration::{
            application_status, client_status, save_client_config, save_server_config,
        },
        evaluation::{current_evaluation, reset_evaluation, start_evaluation, stop_application},
        events::{evaluation_events, transfer_event, transfer_events},
        pages::{client, dashboard, results, server},
        results::{results_data, results_pdf},
    },
    state::DashboardState,
};

use axum::{
    Router,
    routing::{get, post},
};

use http::{HeaderValue, Method};

use std::sync::Arc;

use tower_http::{cors::CorsLayer, services::ServeDir};

/// Creates the HTTP router for the dashboard.
///
/// The router connects each HTTP endpoint with its corresponding
/// handler and configures CORS, shared application state, and
/// static file serving.
pub fn create_router(state: Arc<DashboardState>) -> Router {
    let application_url =
        std::env::var("APPLICATION_URL").unwrap_or_else(|_| "https://127.0.0.1:8443".to_string());

    let application_origin = application_url
        .parse::<HeaderValue>()
        .expect("Invalid APPLICATION_URL");

    let cors = CorsLayer::new()
        .allow_origin(application_origin)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(tower_http::cors::Any);

    Router::new()
        .route("/", get(dashboard))
        .route("/server", get(server))
        .route("/client", get(client))
        .route("/results", get(results))
        .route("/info/server", post(save_server_config))
        .route("/info/client", post(save_client_config))
        .route("/info/client/status", get(client_status))
        .route("/info/application/status", get(application_status))
        .route("/eval/start", post(start_evaluation))
        .route("/eval/current", get(current_evaluation))
        .route("/eval/transfer-event", post(transfer_event))
        .route("/eval/transfer-events", get(transfer_events))
        .route("/eval/evaluation-events", get(evaluation_events))
        .route("/eval/stop", post(stop_application))
        .route("/eval/reset", post(reset_evaluation))
        .route("/results/data", get(results_data))
        .route("/results/pdf", get(results_pdf))
        .nest_service("/static", ServeDir::new("src/dashboard/ui"))
        .with_state(state)
        .layer(cors)
}
