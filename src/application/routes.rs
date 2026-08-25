//! Application HTTP route configuration.
//!
//! This module maps application endpoints to their corresponding
//! handlers and configures static file serving.

use crate::application::handlers::{
    handshake::complete_handshake,
    page::my_app,
    transfer::{
        complete::complete_transfer,
        create::start_app,
        query::{get_latest_transfer, get_transfer},
    },
};
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;

/// Creates the HTTP router for the application service.
///
/// The router exposes endpoints for the application interface,
/// transfer configuration, transfer completion, and TLS handshake
/// metric reporting.
pub fn create_router(pool: sqlx::PgPool) -> Router {
    Router::new()
        .route("/", get(my_app))
        .route("/application/start", post(start_app))
        .route("/transfer/{id}", get(get_transfer))
        .route("/transfer/latest", get(get_latest_transfer))
        .route("/transfer/{id}/complete", post(complete_transfer))
        .route("/handshake/{id}/complete", post(complete_handshake))
        .nest_service("/application-static", ServeDir::new("src/application/ui"))
        .with_state(pool)
}
