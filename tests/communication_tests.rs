use secure_crypt::crypto::algorithm_provider::AlgorithmProvider;
use secure_crypt::crypto::provider::classical_provider::ClassicalProvider;
use secure_crypt::crypto::provider::post_quantum_provider::PostQuantumProvider;

#[test]
fn classical_provider_builds_selected_crypto_provider() {
    let provider = ClassicalProvider::new();

    let cipher_suites = vec!["TLS13_AES_128_GCM_SHA256".to_string()];
    let kx_groups = vec!["X25519".to_string()];

    let crypto_provider =
        provider.build_crypto_provider(&cipher_suites, &kx_groups);

    assert!(
        crypto_provider
            .cipher_suites
            .iter()
            .any(|suite| format!("{:?}", suite.suite()) == "TLS13_AES_128_GCM_SHA256"),
        "The classical cipher suite should be selected"
    );

    assert!(
        crypto_provider
            .kx_groups
            .iter()
            .any(|group| format!("{:?}", group.name()) == "X25519"),
        "The classical key exchange group should be selected"
    );
}

#[test]
fn post_quantum_provider_builds_selected_crypto_provider() {
    let provider = PostQuantumProvider::new();

    let cipher_suites = vec!["TLS13_AES_128_GCM_SHA256".to_string()];
    let kx_groups = vec!["MLKEM768".to_string()];

    let crypto_provider =
        provider.build_crypto_provider(&cipher_suites, &kx_groups);

    assert!(
        crypto_provider
            .cipher_suites
            .iter()
            .any(|suite| format!("{:?}", suite.suite()) == "TLS13_AES_128_GCM_SHA256"),
        "The post-quantum cipher suite should be selected"
    );

    assert!(
        crypto_provider
            .kx_groups
            .iter()
            .any(|group| format!("{:?}", group.name()) == "MLKEM768"),
        "The post-quantum key exchange group should be selected"
    );
}

#[test]
fn classical_provider_reports_correct_name() {
    let provider = ClassicalProvider::new();

    assert_eq!(provider.name(), "classical");
}

#[test]
fn post_quantum_provider_reports_correct_name() {
    let provider = PostQuantumProvider::new();

    assert_eq!(provider.name(), "post_quantum");
}