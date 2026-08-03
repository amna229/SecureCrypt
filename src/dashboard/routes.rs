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
        start_business_logic
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
        .route("/bl/start", post(start_business_logic))
        .nest_service("/static", ServeDir::new("src/dashboard/ui"))
        .with_state(state)

}