use rustls::CipherSuite;

pub fn supported_cipher_suites() -> &'static [CipherSuite] {
    &[
        CipherSuite::TLS13_AES_128_GCM_SHA256,
        CipherSuite::TLS13_AES_256_GCM_SHA384,
        CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
        CipherSuite::TLS13_AES_128_CCM_SHA256,
        CipherSuite::TLS13_AES_128_CCM_8_SHA256,
    ]
}





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classical_and_tls13_cipher_suites_are_available() {
        let suites = supported_cipher_suites();

        assert!(
            suites.contains(&rustls::CipherSuite::TLS13_AES_128_GCM_SHA256)
        );

        assert!(
            suites.contains(&rustls::CipherSuite::TLS13_AES_256_GCM_SHA384)
        );
    }
}