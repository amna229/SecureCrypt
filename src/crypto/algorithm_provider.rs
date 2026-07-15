use rustls::{ClientConfig, ServerConfig};



pub trait AlgorithmProvider {

    fn build_client_config(&self) -> Result<ClientConfig, Box<dyn std::error::Error>>;
    fn build_server_config(&self) -> Result<ServerConfig, Box<dyn std::error::Error>>;

}