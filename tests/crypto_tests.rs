use secure_crypt::crypto::algorithm_provider::AlgorithmProvider;
use secure_crypt::crypto::config::CryptoConfig;
use secure_crypt::crypto::crypto_mode::CryptoMode;
use secure_crypt::crypto::crypto_selector::get_crypto_provider;
use secure_crypt::crypto::provider::classical_provider::ClassicalProvider;
use secure_crypt::crypto::provider::post_quantum_provider::PostQuantumProvider;

#[test]
fn classical_crypto_configuration_is_created() {
    let config = CryptoConfig::new(
        CryptoMode::new("classical".to_string()),
        vec!["TLS13_AES_128_GCM_SHA256".to_string()],
        vec!["X25519".to_string()],
    );

    assert_eq!(config.crypto_mode.to_string(), "classical");
    assert_eq!(config.cipher_suites.len(), 1);
    assert_eq!(config.kx_groups.len(), 1);
}

#[test]
fn post_quantum_crypto_configuration_is_created() {
    let config = CryptoConfig::new(
        CryptoMode::new("post_quantum".to_string()),
        vec!["TLS13_AES_128_GCM_SHA256".to_string()],
        vec!["MLKEM768".to_string()],
    );

    assert_eq!(config.crypto_mode.to_string(), "post_quantum");
    assert_eq!(config.cipher_suites.len(), 1);
    assert_eq!(config.kx_groups.len(), 1);
}

#[test]
fn classical_provider_is_selected() {
    let mode = CryptoMode::new("classical".to_string());

    let provider = get_crypto_provider(&mode)
        .expect("The classical provider should be selected");

    assert_eq!(provider.name(), "classical");
}

#[test]
fn post_quantum_provider_is_selected() {
    let mode = CryptoMode::new("post_quantum".to_string());

    let provider = get_crypto_provider(&mode)
        .expect("The post-quantum provider should be selected");

    assert_eq!(provider.name(), "post_quantum");
}

#[test]
fn unknown_crypto_mode_is_rejected() {
    let mode = CryptoMode::new("invalid".to_string());

    let result = get_crypto_provider(&mode);

    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        "Unknown crypto provider: invalid"
    );
}

#[test]
fn classical_tls_configurations_are_created() {
    let provider = ClassicalProvider::new();

    let cipher_suites = vec!["TLS13_AES_128_GCM_SHA256".to_string()];
    let kx_groups = vec!["X25519".to_string()];

    let client_result = provider.build_client_config(&cipher_suites, &kx_groups);
    let server_result = provider.build_server_config(&cipher_suites, &kx_groups);

    assert!(
        client_result.is_ok(),
        "The classical client TLS configuration should be created successfully"
    );

    assert!(
        server_result.is_ok(),
        "The classical server TLS configuration should be created successfully"
    );
}

#[test]
fn post_quantum_tls_configurations_are_created() {
    let provider = PostQuantumProvider::new();

    let cipher_suites = vec!["TLS13_AES_128_GCM_SHA256".to_string()];
    let kx_groups = vec!["MLKEM768".to_string()];

    let client_result = provider.build_client_config(&cipher_suites, &kx_groups);
    let server_result = provider.build_server_config(&cipher_suites, &kx_groups);

    assert!(
        client_result.is_ok(),
        "The post-quantum client TLS configuration should be created successfully"
    );

    assert!(
        server_result.is_ok(),
        "The post-quantum server TLS configuration should be created successfully"
    );
}