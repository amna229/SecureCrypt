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
