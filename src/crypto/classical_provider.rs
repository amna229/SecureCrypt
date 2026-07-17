// Implementación de AlgorithmProvider para criptografía clásica.
//
// Proporciona la configuración TLS necesaria para utilizar
// mecanismos criptográficos clásicos.

use crate::crypto::algorithm_provider::AlgorithmProvider;
use rustls::{ClientConfig, ServerConfig};

pub struct ClassicalProvider;

//devuelve instancia de ClassicalProvider
impl ClassicalProvider {
    pub fn new() -> Self {
        ClassicalProvider
    }
}

impl AlgorithmProvider for ClassicalProvider {
    fn build_client_config(&self) -> Result<ClientConfig, Box<dyn std::error::Error>> {
        println!("Usando proveedor clásico");
        todo!("Implement the build_client_config method for ClassicalProvider");
    }

    fn build_server_config(&self) -> Result<ServerConfig, Box<dyn std::error::Error>> {
        println!("Usando proveedor clásico");
        todo!("Implement the build_server_config method for ClassicalProvider");
    }
}
