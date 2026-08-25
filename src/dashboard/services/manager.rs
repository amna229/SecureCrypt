//! Dashboard service manager.
//!
//! This module defines the manager responsible for coordinating the
//! dashboard services and providing access to shared application state
//! and Docker resources.

use crate::dashboard::state::DashboardState;

use bollard::Docker;

use std::sync::Arc;

/// Coordinates the services required to manage an evaluation environment.
///
/// The manager provides shared access to the dashboard state and the Docker
/// client. Evaluation lifecycle operations and Docker-specific operations are
/// implemented in separate modules to keep responsibilities isolated.
pub struct Manager {
    pub(crate) state: Arc<DashboardState>,
    pub(crate) docker: Docker,
}

impl Manager {
    /// Creates a new evaluation manager.
    ///
    /// Establishes a connection with the local Docker daemon and stores
    /// a shared reference to the dashboard state.
    pub fn new(state: Arc<DashboardState>) -> Self {
        let docker = Docker::connect_with_local_defaults().expect("Cannot connect to Docker");

        Self { state, docker }
    }
}
