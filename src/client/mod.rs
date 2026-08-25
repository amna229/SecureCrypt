use crate::crypto::CryptoConfig;
use crate::crypto::tls::connect_tls;

use std::error::Error;

type BoxError = Box<dyn Error + Send + Sync>;

/// Establishes a TLS connection using the SecureCrypt library.
///
/// This client is intentionally independent from the application
/// protocol. The returned TLS stream can be consumed by HTTP/1.1,
/// gRPC or another application-level protocol.
pub async fn run_client(
    crypto_config: CryptoConfig,
    addr: &str,
    domain: &str,
) -> Result<(), BoxError> {
    let (_tls_stream, handshake_duration_ms, kx_group, cipher_suite) =
        connect_tls(&crypto_config, addr, domain)
            .await
            .map_err(|(error, _)| error)?;

    println!("TLS connection established with {}", addr);

    println!(
        "TLS handshake: {} ms | KX: {:?} | Cipher suite: {:?}",
        handshake_duration_ms, kx_group, cipher_suite
    );

    Ok(())
}
