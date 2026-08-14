use crate::dashboard::{
    handlers::{
        application_status, client, client_status, dashboard, results, save_client_config,
        save_server_config, server, start_application, start_evaluation, stop_application,
    },
    state::DashboardState,
};
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;
use tower_http::services::ServeDir;

pub fn create_router(state: Arc<DashboardState>) -> Router {
    Router::new()
        .route("/", get(dashboard))
        .route("/server", get(server))
        .route("/client", get(client))
        .route("/results", get(results))
        .route("/info/server", post(save_server_config))
        .route("/info/client", post(save_client_config))
        .route("/eval/start", post(start_evaluation))
        .route("/info/client/status", get(client_status))
        .route("/info/application/status", get(application_status))
        .route("/application/start", post(start_application))
        .route("/application/stop", post(stop_application))
        .nest_service("/static", ServeDir::new("src/dashboard/ui"))
        .with_state(state)
}
