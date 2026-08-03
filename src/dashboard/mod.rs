pub mod handlers;
pub mod routes;
pub mod services;
pub mod state;
pub mod ui;

use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use crate::dashboard::routes::create_router;
use crate::dashboard::state::DashboardState;



pub async fn run_dashboard() -> Result<(), Box<dyn Error + Send + Sync>>{

    let state = Arc::new(DashboardState::new());

    let app = create_router(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    println!("Dashboard at http://127.0.0.1:3000");

    axum::serve(listener, app).await?;

    Ok(())

}