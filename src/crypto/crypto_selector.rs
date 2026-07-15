// Selecciona el proveedor criptográfico solicitado por el usuario.
//
// Recibe el mecanismo elegido desde main (o desde el dashboard)
// y devuelve la implementación correspondiente de AlgorithmProvider.
//
// No conoce cliente, servidor ni Tokio.

use crate::crypto::algorithm_provider::AlgorithmProvider;
use crate::crypto::classical_provider::ClassicalProvider;
use crate::crypto::post_quantum_provider::PostQuantumProvider;
use crate::crypto::crypto_mode::CryptoMode;



pub fn get_crypto_provider(mode: CryptoMode) -> Box<dyn AlgorithmProvider> {

    match mode {

        CryptoMode::Classical => Box::new(ClassicalProvider::new()),
        CryptoMode::PostQuantum => Box::new(PostQuantumProvider::new()),

    }
}