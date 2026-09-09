//! gRPC client for the chat application.
//!
//! This module contains the application-level client used to send
//! messages through a connection provided by SecureCrypt.

use crate::server::chat::{ChatMessage, chat_service_client::ChatServiceClient};
use crate::transport::{BoxedApplicationWrapper, SecureCryptConnector};

use tonic::transport::Endpoint;

/// Sends a message through an existing SecureCrypt connection.
pub async fn send_message(
    stream: BoxedApplicationWrapper,
    client_id: String,
    content: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Sending message from {}: {}", client_id, content);

    // The connection has already been established by SecureCrypt.
    // We only adapt the provided stream so that Tonic can use it.
    let connector = SecureCryptConnector {
        stream: Some(stream),
    };

    let endpoint = Endpoint::from_static("http://chat");

    let channel = endpoint.connect_with_connector(connector).await?;

    let mut client = ChatServiceClient::new(channel);

    let request = tonic::Request::new(ChatMessage { client_id, content });

    let response = client.send_message(request).await?;

    println!("Server response: {}", response.into_inner().message);

    Ok(())
}
