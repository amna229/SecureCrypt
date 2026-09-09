//! gRPC server for the external application.
//!
//! This module contains the application-level gRPC service and the
//! integration point with the SecureCrypt server API.

use secure_crypt::user_application::BoxedApplicationStream;

use std::error::Error;
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Server;

use crate::transport::BoxedApplicationWrapper;

pub mod chat {
    tonic::include_proto!("chat");
}

/// Application-level gRPC chat service.
pub struct ChatService;

#[tonic::async_trait]
impl chat::chat_service_server::ChatService for ChatService {
    async fn send_message(
        &self,
        request: tonic::Request<chat::ChatMessage>,
    ) -> Result<tonic::Response<chat::MessageResponse>, tonic::Status> {
        let message = request.into_inner();

        println!("[{}] {}", message.client_id, message.content);

        Ok(tonic::Response::new(chat::MessageResponse {
            success: true,
            message: format!("Message received from {}", message.client_id),
        }))
    }
}

/// Starts the gRPC server using connections provided by SecureCrypt.
pub async fn run_server() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (tx, rx) = mpsc::channel::<Result<BoxedApplicationWrapper, std::io::Error>>(100);

    let incoming = ReceiverStream::new(rx);

    tokio::spawn(async move {
        Server::builder()
            .add_service(chat::chat_service_server::ChatServiceServer::new(
                ChatService,
            ))
            .serve_with_incoming(incoming)
            .await
            .expect("gRPC server failed");
    });

    let tx_for_application = Arc::new(tx);

    // SecureCrypt is responsible for:
    // - receiving the evaluation configuration,
    // - configuring TLS/PQC,
    // - establishing the secure connection,
    // - exposing the resulting stream to the application.
    secure_crypt::server::start_application(Arc::new(move |stream: BoxedApplicationStream| {
        let tx = Arc::clone(&tx_for_application);

        async move {
            tx.send(Ok(BoxedApplicationWrapper { inner: stream }))
                .await
                .map_err(|error| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to send stream: {}", error),
                    )
                })?;

            Ok(())
        }
    }))
    .await?;

    Ok(())
}





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::chat::chat_service_server::ChatService as ChatServiceTrait;

    #[tokio::test]
    async fn chat_service_accepts_and_processes_a_message() {
        let service = ChatService;

        let request = tonic::Request::new(chat::ChatMessage {
            client_id: "client-01".to_string(),
            content: "Hello SecureCrypt".to_string(),
        });

        let response = service
            .send_message(request)
            .await
            .expect("The gRPC message should be processed successfully");

        let message = response.into_inner();

        assert!(message.success);
        assert_eq!(message.message, "Message received from client-01");
    }
}