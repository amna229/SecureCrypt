use secure_crypt::crypto::tls::{
    accept_tls_connection,
    connect_tls,
    create_tls_server,
};
use secure_crypt::crypto::{CryptoConfig, CryptoMode};
use tokio::time::{timeout, Duration};

const TEST_ADDR: &str = "127.0.0.1:18443";
const TEST_DOMAIN: &str = "localhost";

#[tokio::test]
async fn classical_tls_handshake_is_established() {
    let config = CryptoConfig::new(
        CryptoMode::new("classical".to_string()),
        vec!["TLS13_AES_128_GCM_SHA256".to_string()],
        vec!["X25519".to_string()],
    );

    let (listener, acceptor) = create_tls_server(&config, TEST_ADDR)
        .await
        .expect("The classical TLS server should start successfully");

    let server_task = tokio::spawn(async move {
        let (stream, _) = listener
            .accept()
            .await
            .expect("The server should accept the TCP connection");

        accept_tls_connection(&acceptor, stream)
            .await
            .expect("The classical TLS handshake should succeed");
    });

    let client_result = timeout(
        Duration::from_secs(5),
        connect_tls(&config, TEST_ADDR, TEST_DOMAIN),
    )
    .await
    .expect("The classical TLS handshake should finish within the timeout");

    let (_, _, kx_group, cipher_suite) = client_result
        .expect("The classical TLS connection should be established successfully");

    assert_eq!(kx_group.as_deref(), Some("X25519"));
    assert_eq!(
        cipher_suite.as_deref(),
        Some("TLS13_AES_128_GCM_SHA256")
    );

    server_task
        .await
        .expect("The classical TLS server task should finish successfully");
}

#[tokio::test]
async fn post_quantum_tls_handshake_is_established() {
    let config = CryptoConfig::new(
        CryptoMode::new("post_quantum".to_string()),
        vec!["TLS13_AES_128_GCM_SHA256".to_string()],
        vec!["MLKEM768".to_string()],
    );

    let (listener, acceptor) = create_tls_server(&config, "127.0.0.1:18444")
        .await
        .expect("The post-quantum TLS server should start successfully");

    let server_task = tokio::spawn(async move {
        let (stream, _) = listener
            .accept()
            .await
            .expect("The server should accept the TCP connection");

        accept_tls_connection(&acceptor, stream)
            .await
            .expect("The post-quantum TLS handshake should succeed");
    });

    let client_result = timeout(
        Duration::from_secs(5),
        connect_tls(
            &config,
            "127.0.0.1:18444",
            TEST_DOMAIN,
        ),
    )
    .await
    .expect("The post-quantum TLS handshake should finish within the timeout");

    let (_, _, kx_group, cipher_suite) = client_result
        .expect("The post-quantum TLS connection should be established successfully");

    assert_eq!(kx_group.as_deref(), Some("MLKEM768"));
    assert_eq!(
        cipher_suite.as_deref(),
        Some("TLS13_AES_128_GCM_SHA256")
    );

    server_task
        .await
        .expect("The post-quantum TLS server task should finish successfully");
}