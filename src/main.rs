//! SecureCrypt chat application.
//!
//! This binary represents the external application used by the
//! SecureCrypt evaluation environment.
//!
//! When no ROLE is provided, the application starts the SecureCrypt
//! dashboard. The dashboard subsequently launches this same executable
//! with ROLE=server or ROLE=client for evaluation processes.

mod client;
mod server;
mod transport;

use secure_crypt::client::start_application;
use secure_crypt::dashboard::run_dashboard;

use std::env;
use std::error::Error;

use transport::BoxedApplicationWrapper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Starting SecureCrypt chat application...");

    let role = env::var("ROLE").unwrap_or_else(|_| "dashboard".to_string());

    match role.as_str() {
        "dashboard" => {
            println!("Starting SecureCrypt dashboard...");

            let application_program = env::current_exe()?;

            run_dashboard(application_program.to_string_lossy().into_owned()).await?;
        }

        "server" => {
            println!("Starting chat server...");

            server::run_server().await?;
        }

        "client" => {
            println!("Starting chat client...");

            start_application(|stream| async move {
                let stream = BoxedApplicationWrapper { inner: stream };

                let client_id = env::var("CLIENT_ID").unwrap_or_else(|_| "1".to_string());

                client::send_message(
                    stream,
                    format!("Client {}", client_id),
                    "Hello, Server!".into(),
                )
                .await?;

                Ok(())
            })
            .await?;
        }

        _ => {
            return Err("Invalid ROLE. Expected 'dashboard', 'server' or 'client'".into());
        }
    }

    println!("SecureCrypt chat application finished");

    Ok(())
}
