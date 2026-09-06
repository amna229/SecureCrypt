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





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn application_config_is_created_correctly() {
        let evaluation_id = Uuid::new_v4();

        let config = ApplicationConfig {
            evaluation_id,
            operation: "upload".to_string(),
            file_size: 10,
            file_size_unit: "MB".to_string(),
            num_files: 2,
        };

        assert_eq!(config.evaluation_id, evaluation_id);
        assert_eq!(config.operation, "upload");
        assert_eq!(config.file_size, 10);
        assert_eq!(config.file_size_unit, "MB");
        assert_eq!(config.num_files, 2);
    }

    #[test]
    fn application_config_serializes_and_deserializes_correctly() {
        let evaluation_id = Uuid::new_v4();

        let config = ApplicationConfig {
            evaluation_id,
            operation: "download".to_string(),
            file_size: 20,
            file_size_unit: "MB".to_string(),
            num_files: 3,
        };

        let serialized = serde_json::to_string(&config)
            .expect("Application configuration should serialize successfully");

        let deserialized: ApplicationConfig = serde_json::from_str(&serialized)
            .expect("Application configuration should deserialize successfully");

        assert_eq!(deserialized.evaluation_id, evaluation_id);
        assert_eq!(deserialized.operation, "download");
        assert_eq!(deserialized.file_size, 20);
        assert_eq!(deserialized.file_size_unit, "MB");
        assert_eq!(deserialized.num_files, 3);
    }
}