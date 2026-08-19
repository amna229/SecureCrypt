#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct CryptoMode(pub String);

impl CryptoMode {
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for CryptoMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
