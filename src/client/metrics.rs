use uuid::Uuid;

/// Metrics reported after a file transfer.
///
/// These metrics are collected by the client and sent to the
/// application so that the performance of the transfer can be evaluated.
#[derive(Debug, serde::Serialize)]
pub struct CompleteTransferRequest {
    /// Total transfer duration in milliseconds.
    pub duration_ms: i64,

    /// Total number of bytes transferred.
    pub bytes_transferred: i64,

    /// Transfer throughput in megabits per second.
    pub throughput_mbps: f64,

    /// Cryptographic mode used during the transfer.
    pub crypto_mode: String,

    /// Key exchange group negotiated during the TLS handshake.
    pub kx_group: String,

    /// Cipher suite negotiated during the TLS handshake.
    pub cipher_suite: String,

    /// Indicates whether the transfer completed successfully.
    pub success: bool,

    /// Description of the error if the transfer failed.
    pub error_type: Option<String>,
}

/// Metrics reported after a TLS handshake.
///
/// These metrics are used to evaluate the cost of establishing
/// a secure connection using the selected cryptographic mechanism.
#[derive(Debug, serde::Serialize)]
pub struct CompleteHandshakeRequest {
    /// Identifier of the evaluation to which the handshake belongs.
    pub evaluation_id: Uuid,

    /// Identifier of the client that performed the handshake.
    pub client_id: i32,

    /// Time required to complete the TLS handshake in milliseconds.
    pub handshake_duration_ms: i64,

    /// Key exchange group negotiated during the handshake.
    pub kx_group: Option<String>,

    /// Cipher suite negotiated during the handshake.
    pub cipher_suite: Option<String>,

    /// Indicates whether the handshake completed successfully.
    pub success: bool,
}





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn transfer_metrics_serialize_correctly() {
        let metrics = CompleteTransferRequest {
            duration_ms: 100,
            bytes_transferred: 2048,
            throughput_mbps: 1.5,
            crypto_mode: "classical".to_string(),
            kx_group: "X25519".to_string(),
            cipher_suite: "TLS13_AES_128_GCM_SHA256".to_string(),
            success: true,
            error_type: None,
        };

        let json =
            serde_json::to_value(&metrics).expect("Transfer metrics should serialize correctly");

        assert_eq!(json["duration_ms"], Value::from(100));
        assert_eq!(json["bytes_transferred"], Value::from(2048));
        assert_eq!(json["crypto_mode"], Value::from("classical"));
        assert_eq!(json["success"], Value::from(true));
    }

    #[test]
    fn handshake_metrics_serialize_correctly() {
        let evaluation_id = Uuid::new_v4();

        let metrics = CompleteHandshakeRequest {
            evaluation_id,
            client_id: 1,
            handshake_duration_ms: 25,
            kx_group: Some("X25519".to_string()),
            cipher_suite: Some("TLS13_AES_128_GCM_SHA256".to_string()),
            success: true,
        };

        let json =
            serde_json::to_value(&metrics).expect("Handshake metrics should serialize correctly");

        assert_eq!(json["evaluation_id"], evaluation_id.to_string());
        assert_eq!(json["client_id"], Value::from(1));
        assert_eq!(json["handshake_duration_ms"], Value::from(25));
        assert_eq!(json["success"], Value::from(true));
    }
}