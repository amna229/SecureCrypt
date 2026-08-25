//! Dashboard application.
//!
//! This module initializes the dashboard database connection, shared state,
//! HTTP router, and asynchronous HTTP server.

pub mod database;
pub mod handlers;
pub mod metrics;
pub mod routes;
pub mod services;
pub mod state;
pub mod ui;

use crate::dashboard::database::create_pool;
use crate::dashboard::routes::create_router;
use crate::dashboard::state::DashboardState;

use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Starts the dashboard HTTP server.
///
/// The function initializes the database connection pool, creates the
/// shared dashboard state, builds the HTTP router, and starts listening
/// for incoming HTTP requests.
pub async fn run_dashboard() -> Result<(), Box<dyn Error + Send + Sync>> {
    let db = create_pool().await?;

    let state = Arc::new(DashboardState::new(db));

    let app = create_router(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    println!("Dashboard at http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
