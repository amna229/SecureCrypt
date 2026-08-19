#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApplicationConfig {
    pub operation: String,
    pub file_size: u64,
    pub file_size_unit: String,
    pub num_files: u32,
}
