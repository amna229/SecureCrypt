/// Represents the cryptographic mode selected for an evaluation.
#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct CryptoMode(pub String);

impl CryptoMode {
    /// Creates a new cryptographic mode from its identifier.
    pub fn new(value: String) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for CryptoMode {
    /// Formats the cryptographic mode as its identifier.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
