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

use crate::dashboard::database::create_storage;
use crate::dashboard::routes::create_router;
use crate::dashboard::state::DashboardState;

use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Starts the dashboard HTTP server.
///
/// The executable path is provided by the application embedding the
/// SecureCrypt library and stored in the shared dashboard state.
pub async fn run_dashboard(
    application_program: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let storage = create_storage().await;

    let state = Arc::new(DashboardState::new(storage));

    state.set_application_program(application_program).await;

    let app = create_router(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;

    println!("Dashboard at http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
