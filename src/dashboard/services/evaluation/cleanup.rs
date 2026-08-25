use crate::dashboard::services::manager::Manager;
use crate::dashboard::state::ClientConfig;

impl Manager {
    /// Removes all resources associated with the current evaluation.
    ///
    /// Client containers, the server container, and the evaluation
    /// network are removed. Runtime state is then reset and the
    /// dashboard is notified about the completed cleanup.
    pub(crate) async fn cleanup_evaluation(&self, configs: &[ClientConfig]) -> Result<(), String> {
        let evaluation_id = {
            let evaluation_id = self.state.evaluation_id.lock().await;

            *evaluation_id
        };

        let total_clients = configs
            .iter()
            .map(|config| config.num_connections as i32)
            .sum::<i32>();

        let mut stopped_clients = 0i32;

        for config in configs {
            for client_id in 0..config.num_connections {
                let actual_client_id = client_id + 1;

                let container_name = format!("securecrypt-client-{}", actual_client_id);

                self.remove_client_container(&container_name).await;

                stopped_clients += 1;

                if let Some(evaluation_id) = evaluation_id {
                    self.send_evaluation_event(
                        "client-stopped",
                        evaluation_id,
                        Some(actual_client_id as i32),
                        stopped_clients as i64,
                        total_clients as i64,
                    );
                }
            }
        }

        self.remove_server_container().await;

        if let Some(evaluation_id) = evaluation_id {
            self.send_evaluation_event(
                "server-stopped",
                evaluation_id,
                None,
                stopped_clients as i64,
                total_clients as i64,
            );
        }

        self.disconnect_application_from_evaluation_network().await;

        self.remove_evaluation_network().await;

        {
            let mut application_running = self.state.application_running.lock().await;

            *application_running = false;
        }

        {
            let mut is_running = self.state.is_evaluation_running.lock().await;

            *is_running = false;
        }

        if let Some(evaluation_id) = evaluation_id {
            self.send_evaluation_event(
                "environment-stopped",
                evaluation_id,
                None,
                total_clients as i64,
                total_clients as i64,
            );
        }

        {
            let mut current_evaluation_id = self.state.evaluation_id.lock().await;

            *current_evaluation_id = None;
        }

        println!("Evaluation environment cleaned up");

        Ok(())
    }
}
