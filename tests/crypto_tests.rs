use secure_crypt::crypto::crypto_mode::CryptoMode;
use secure_crypt::crypto::crypto_selector::get_crypto_provider;

#[test]
fn crypto_mode_classical_is_created_correctly() {
    let mode = CryptoMode::new("classical".to_string());

    assert_eq!(mode.0, "classical");
    assert_eq!(mode.to_string(), "classical");
}

#[test]
fn crypto_mode_post_quantum_is_created_correctly() {
    let mode = CryptoMode::new("post_quantum".to_string());

    assert_eq!(mode.0, "post_quantum");
    assert_eq!(mode.to_string(), "post_quantum");
}

#[test]
fn classical_provider_is_selected() {
    let mode = CryptoMode::new("classical".to_string());

    let provider = get_crypto_provider(&mode)
        .expect("Classical provider should be selected");

    assert_eq!(provider.name(), "classical");
}

#[test]
fn post_quantum_provider_is_selected() {
    let mode = CryptoMode::new("post_quantum".to_string());

    let provider = get_crypto_provider(&mode)
        .expect("Post-quantum provider should be selected");

    assert_eq!(provider.name(), "post_quantum");
}

#[test]
fn unknown_provider_returns_error() {
    let mode = CryptoMode::new("invalid".to_string());

    let result = get_crypto_provider(&mode);

    assert!(result.is_err());
    assert_eq!(
        result.err().unwrap(),
        "Unknown crypto provider: invalid"
    );
}