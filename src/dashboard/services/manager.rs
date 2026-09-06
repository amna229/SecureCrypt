//! Dashboard service manager.

use crate::dashboard::state::DashboardState;

use bollard::Docker;

use std::sync::Arc;

pub struct Manager {
    pub(crate) state: Arc<DashboardState>,
    pub(crate) docker: Option<Docker>,
}

impl Manager {
    /// Creates a new evaluation manager.
    ///
    /// Docker is optional because SecureCrypt can operate in standalone
    /// mode without a Docker daemon.
    pub fn new(state: Arc<DashboardState>) -> Self {
        let docker = Docker::connect_with_local_defaults().ok();

        if docker.is_some() {
            println!("Docker connection available");
        } else {
            println!("Docker unavailable: standalone mode");
        }

        Self { state, docker }
    }
}
