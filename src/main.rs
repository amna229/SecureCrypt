mod client;
mod crypto;
mod server;
mod dashboard;
mod application;
use std::error::Error;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    dashboard::run_dashboard().await?;
    Ok(())

}