use uuid::Uuid;

#[derive(Debug, serde::Deserialize)]
pub struct TransferConfig {
    pub transfer_id: Uuid,
    pub operation: String,
    pub file_size_bytes: u64,
    pub num_files: u32,
    pub created_at: i64,
}
