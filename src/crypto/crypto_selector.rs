use crate::crypto::algorithm_provider::AlgorithmProvider;
use crate::crypto::crypto_mode::CryptoMode;
use crate::crypto::provider::classical_provider::ClassicalProvider;
use crate::crypto::provider::post_quantum_provider::PostQuantumProvider;

/// Selects the cryptographic provider corresponding to the requested mode.
///
/// The selector hides the concrete provider implementations from the
/// rest of the application and returns them through the `AlgorithmProvider`
/// abstraction.
pub fn get_crypto_provider(mode: &CryptoMode) -> Result<Box<dyn AlgorithmProvider>, String> {
    match mode.0.as_str() {
        "classical" => Ok(Box::new(ClassicalProvider::new())),
        "post_quantum" => Ok(Box::new(PostQuantumProvider::new())),
        other => Err(format!("Unknown crypto provider: {}", other)),
    }
}





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::crypto_mode::CryptoMode;

    #[test]
    fn selects_classical_provider() {
        let mode = CryptoMode::new("classical".to_string());

        let provider = get_crypto_provider(&mode)
            .expect("Classic provider should be selected");

        assert_eq!(provider.name(), "classical");
    }

    #[test]
    fn selects_post_quantum_provider() {
        let mode = CryptoMode::new("post_quantum".to_string());

        let provider = get_crypto_provider(&mode)
            .expect("Post-quantum provider should be selected");

        assert_eq!(provider.name(), "post_quantum");
    }

    #[test]
    fn rejects_unknown_provider() {
        let mode = CryptoMode::new("invalid".to_string());

        let result = get_crypto_provider(&mode);

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            "Unknown crypto provider: invalid"
        );
    }
}