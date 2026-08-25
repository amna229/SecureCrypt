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
