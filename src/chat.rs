//! gRPC chat service.
//!
//! This module contains the application-level logic of the chat.
//! SecureCrypt is responsible for providing the secure communication
//! channel used by this service.

pub mod proto {
    tonic::include_proto!("chat");
}

/// Application-level gRPC chat service.
pub struct ChatService;

#[tonic::async_trait]
impl proto::chat_service_server::ChatService for ChatService {
    async fn send_message(
        &self,
        request: tonic::Request<proto::ChatMessage>,
    ) -> Result<tonic::Response<proto::MessageResponse>, tonic::Status> {
        let message = request.into_inner();

        println!("[{}] {}", message.client_id, message.content);

        Ok(tonic::Response::new(proto::MessageResponse {
            success: true,
            message: format!("Message received from {}", message.client_id),
        }))
    }
}
