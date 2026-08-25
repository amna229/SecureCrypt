mod client;
mod crypto;
mod dashboard;
mod server;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dashboard::run_dashboard().await?;
    Ok(())
}
