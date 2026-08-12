use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;



#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerConfig {

    pub key_exchange: crate::crypto::crypto_mode::CryptoMode,
    pub cipher_suites: Vec<String>,
    pub kx_groups: Vec<String>


}



#[derive(Debug, Clone, serde::Deserialize)]
pub struct ClientConfig {

    pub key_exchange: crate::crypto::crypto_mode::CryptoMode,
    pub cipher_suites: Vec<String>,
    pub kx_groups: Vec<String>,
    pub num_connections: u32

}



pub struct DashboardState {

    pub client_configs: Mutex<Vec<ClientConfig>>,
    pub server_config: Mutex<Option<ServerConfig>>,
    pub is_evaluation_running: Mutex<bool>,
    pub application_running: Mutex<bool>

}



impl DashboardState {

    pub fn new() -> Self {

        Self {

            client_configs: Mutex::new(Vec::new()),
            server_config: Mutex::new(None),
            is_evaluation_running: Mutex::new(false),
            application_running: Mutex::new(false)
            
        }
    }



    pub async fn is_evaluation_running(&self) -> bool {

        *self.is_evaluation_running.lock().await
        
    }
}