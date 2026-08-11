use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use crate::dashboard::{
    handlers::{
        client,
        dashboard,
        results,
        save_client_config,
        save_server_config,
        server,
        start_evaluation,
        client_status,
        application_status,
        start_application
    },
    state::DashboardState
};
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
        .route("/info/client/status",get(client_status))
        .route("/info/application/status", get(application_status))
        .route("/application/start", post(start_application))
        .nest_service("/static", ServeDir::new("src/dashboard/ui"))
        .with_state(state)

}