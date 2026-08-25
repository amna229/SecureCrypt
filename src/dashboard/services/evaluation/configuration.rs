use crate::dashboard::services::manager::Manager;
use crate::dashboard::state::{ClientConfig, ServerConfig};

impl Manager {
    /// Retrieves the current server configuration from the dashboard state.
    ///
    /// Returns an error if no server configuration has been defined.
    pub(crate) async fn get_server_config(&self) -> Result<ServerConfig, String> {
        let config = self.state.server_config.lock().await;

        config
            .clone()
            .ok_or_else(|| "Server configuration not found".to_string())
    }

    /// Retrieves all configured client settings from the dashboard state.
    ///
    /// Returns an error if no client configuration has been defined.
    pub(crate) async fn get_client_configs(&self) -> Result<Vec<ClientConfig>, String> {
        let configs = self.state.client_configs.lock().await;

        if configs.is_empty() {
            return Err("No client configuration found".to_string());
        }

        Ok(configs.clone())
    }

    /// Resets the current evaluation configuration.
    ///
    /// This also stops any active resource monitoring and clears
    /// the current evaluation identifier.
    pub async fn reset_configuration(&self) {
        {
            let token = self.state.resource_monitor_token.lock().await;

            token.cancel();
        }

        let mut monitor = self.state.resource_monitor.lock().await;

        if let Some(resource_monitor) = monitor.take() {
            resource_monitor.stop().await;
        }

        drop(monitor);

        let mut server_config = self.state.server_config.lock().await;

        *server_config = None;

        drop(server_config);

        let mut client_configs = self.state.client_configs.lock().await;

        client_configs.clear();

        drop(client_configs);

        let mut evaluation_id = self.state.evaluation_id.lock().await;

        *evaluation_id = None;

        println!("Evaluation configuration reset");
    }
}
