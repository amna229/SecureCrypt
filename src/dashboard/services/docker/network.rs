use crate::dashboard::services::manager::Manager;

use bollard::models::{
    EndpointSettings, NetworkConnectRequest, NetworkCreateRequest, NetworkDisconnectRequest,
};

impl Manager {
    /// Creates the isolated Docker network used by an evaluation.
    pub(crate) async fn create_evaluation_network(&self) -> Result<(), String> {
        let network_config = NetworkCreateRequest {
            name: "securecrypt-evaluation-network".to_string(),
            ..Default::default()
        };

        self.docker
            .create_network(network_config)
            .await
            .map_err(|e| e.to_string())?;

        println!("Evaluation network created");

        Ok(())
    }

    /// Connects a container to the evaluation network.
    ///
    /// The provided aliases allow containers to resolve each
    /// other through Docker's internal DNS.
    pub(crate) async fn connect_container_to_evaluation_network(
        &self,
        container_name: &str,
        aliases: &[&str],
    ) -> Result<(), String> {
        let connect_request = NetworkConnectRequest {
            container: container_name.to_string(),

            endpoint_config: Some(EndpointSettings {
                aliases: Some(aliases.iter().map(|alias| (*alias).to_string()).collect()),
                ..Default::default()
            }),
        };

        self.docker
            .connect_network("securecrypt-evaluation-network", connect_request)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Connects the main application container to the
    /// evaluation network.
    pub(crate) async fn connect_application_to_evaluation_network(&self) -> Result<(), String> {
        let connect_request = NetworkConnectRequest {
            container: "securecrypt-application".to_string(),

            endpoint_config: Some(EndpointSettings {
                aliases: Some(vec!["securecrypt-application".to_string()]),
                ..Default::default()
            }),
        };

        self.docker
            .connect_network("securecrypt-evaluation-network", connect_request)
            .await
            .map_err(|e| e.to_string())?;

        println!("Application connected to evaluation network");

        Ok(())
    }

    /// Disconnects the application container from
    /// the evaluation network.
    pub(crate) async fn disconnect_application_from_evaluation_network(&self) {
        let disconnect_request = NetworkDisconnectRequest {
            container: "securecrypt-application".to_string(),
            force: Some(true),
        };

        let _ = self
            .docker
            .disconnect_network("securecrypt-evaluation-network", disconnect_request)
            .await;
    }

    /// Removes the Docker network used by the evaluation.
    pub(crate) async fn remove_evaluation_network(&self) {
        let _ = self
            .docker
            .remove_network("securecrypt-evaluation-network")
            .await;
    }
}
