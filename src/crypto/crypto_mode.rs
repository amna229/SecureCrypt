#[derive(Debug, serde::Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CryptoMode {

    Classical,
    PostQuantum,
    
}
