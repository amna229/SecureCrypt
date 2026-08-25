//! Application configuration.
//!
//! This module defines the configuration received from the dashboard
//! when a new transfer is requested.

use uuid::Uuid;

/// Configuration used to create a transfer for an evaluation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApplicationConfig {
    pub evaluation_id: Uuid,
    pub operation: String,
    pub file_size: u64,
    pub file_size_unit: String,
    pub num_files: u32,
}
