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





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_classical_mode() {
        let mode = CryptoMode::new("classical".to_string());

        assert_eq!(mode.0, "classical");
    }

    #[test]
    fn creates_post_quantum_mode() {
        let mode = CryptoMode::new("post_quantum".to_string());

        assert_eq!(mode.0, "post_quantum");
    }

    #[test]
    fn displays_mode_identifier() {
        let mode = CryptoMode::new("classical".to_string());

        assert_eq!(mode.to_string(), "classical");
    }
}