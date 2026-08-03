use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;



#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServerConfig {
    pub key_exchange: crate::crypto::crypto_mode::CryptoMode,
    pub cipher_suites: Vec<String>,
    pub kx_groups: Vec<String>,
}


#[derive(Debug, Clone, serde::Deserialize)]
pub struct ClientConfig {
    pub key_exchange: crate::crypto::crypto_mode::CryptoMode,
    pub cipher_suites: Vec<String>,
    pub kx_groups: Vec<String>,
    pub num_connections: u32,
}


pub struct DashboardState {
    pub server_cancellation_token: Mutex<Option<CancellationToken>>,
    pub client_configs: Mutex<Vec<ClientConfig>>,
    pub server_config: Mutex<Option<ServerConfig>>,
}


impl DashboardState {
    pub fn new() -> Self {
        Self {
            server_cancellation_token: Mutex::new(None),
            client_configs: Mutex::new(Vec::new()),
            server_config: Mutex::new(None),
        }
    }
}