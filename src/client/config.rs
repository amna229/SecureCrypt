use uuid::Uuid;

/// Configuration of a file transfer to be executed by the client.
#[derive(Debug, serde::Deserialize)]
pub struct TransferConfig {
    /// Unique identifier of the transfer configuration.
    pub transfer_id: Uuid,

    /// Operation to perform: `upload` or `download`.
    pub operation: String,

    /// Size of each file in bytes.
    pub file_size_bytes: u64,

    /// Number of files to transfer.
    pub num_files: u32,

    /// Timestamp used to identify when the configuration was created.
    pub created_at: i64,
}





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_config_can_be_deserialized() {
        let json = r#"{
            "transfer_id": "550e8400-e29b-41d4-a716-446655440000",
            "operation": "upload",
            "file_size_bytes": 1024,
            "num_files": 2,
            "created_at": 1234567890
        }"#;

        let config: TransferConfig =
            serde_json::from_str(json).expect("TransferConfig should deserialize successfully");

        assert_eq!(config.operation, "upload");
        assert_eq!(config.file_size_bytes, 1024);
        assert_eq!(config.num_files, 2);
        assert_eq!(config.created_at, 1234567890);
    }
}