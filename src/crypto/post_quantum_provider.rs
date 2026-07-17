// Implementación de AlgorithmProvider para criptografía postcuántica.
//
// Proporciona la configuración TLS necesaria para utilizar
// mecanismos criptográficos postcuánticos.

use crate::crypto::algorithm_provider::AlgorithmProvider;
use rustls::{ClientConfig, ServerConfig};

pub struct PostQuantumProvider;

//devuelve instancia de PostQuantumProvider
impl PostQuantumProvider {
    pub fn new() -> Self {
        PostQuantumProvider
    }
}

impl AlgorithmProvider for PostQuantumProvider {
    fn build_client_config(&self) -> Result<ClientConfig, Box<dyn std::error::Error>> {
        println!("Usando proveedor postcuántico");
        todo!("Implement the build_client_config method for PostQuantumProvider");
    }

    fn build_server_config(&self) -> Result<ServerConfig, Box<dyn std::error::Error>> {
        println!("Usando proveedor postcuántico");
        todo!("Implement the build_server_config method for PostQuantumProvider");
    }
}
