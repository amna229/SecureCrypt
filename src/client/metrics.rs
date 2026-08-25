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
