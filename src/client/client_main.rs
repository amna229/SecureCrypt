//! SecureCrypt client entry point.
//!
//! This binary starts a client using the SecureCrypt library. The library
//! handles the evaluation configuration and TLS establishment, while the
//! application-level protocol is provided through the application callback.

use secure_crypt::client::start_application;

use std::error::Error;

/// Starts the SecureCrypt client.
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("Starting client...");

    start_application(|_tls_stream| async move {
        println!("TLS connection established. Waiting for application protocol...");
        Ok(())
    })
    .await?;

    Ok(())
}
