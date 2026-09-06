//! SecureCrypt client.
//!
//! This module provides the client-side interface for applications using
//! SecureCrypt. It is responsible for obtaining the evaluation configuration,
//! establishing the TLS connection and exposing the resulting stream to the
//! application-level protocol.

use crate::crypto::tls::connect_tls;
use crate::crypto::{CryptoConfig, CryptoMode};
use crate::user_application::{BoxedApplicationStream, MetricsStream};

use reqwest::Client;
use serde_json::json;
use std::env;
use std::error::Error;
use std::future::Future;
use uuid::Uuid;

type BoxError = Box<dyn Error + Send + Sync>;

/// Starts an application over a SecureCrypt TLS client connection.
///
/// SecureCrypt waits for the evaluation configuration provided by the
/// Dashboard, configures the selected cryptographic mechanisms, establishes
/// the TLS connection and exposes a metrics-enabled stream to the application.
///
/// The application is responsible only for implementing its application-level
/// protocol. It does not need to know the concrete TLS implementation or
/// the evaluation metrics.
pub async fn start_application<F, Fut>(application: F) -> Result<(), BoxError>
where
    F: FnOnce(BoxedApplicationStream) -> Fut,
    Fut: Future<Output = Result<(), BoxError>>,
{
    let control_addr =
        env::var("SECURECRYPT_CONTROL_ADDR").unwrap_or_else(|_| "0.0.0.0:9090".to_string());

    let config = crate::control::wait_for_configuration(&control_addr).await?;

    let evaluation_id = config.evaluation_id.clone();

    let server_addr = config
        .server_addr
        .ok_or("Server address not provided in evaluation configuration")?;

    let server_name = config
        .server_name
        .unwrap_or_else(|| "localhost".to_string());

    let crypto_config = CryptoConfig::new(
        CryptoMode::new(config.crypto_mode),
        config.cipher_suites,
        config.kx_groups,
    );

    let (tls_stream, handshake_duration_ms, kx_group, cipher_suite) =
        connect_tls(&crypto_config, &server_addr, &server_name)
            .await
            .map_err(|(error, _)| error)?;

    println!("TLS connection established with {}", server_addr);

    println!(
        "TLS handshake: {} ms | KX: {:?} | Cipher suite: {:?}",
        handshake_duration_ms, kx_group, cipher_suite
    );

    let dashboard_url =
        env::var("DASHBOARD_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".to_string());

    let evaluation_uuid = Uuid::parse_str(&evaluation_id)?;

    let client_id = env::var("CLIENT_ID")
        .ok()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(0);

    let http_client = Client::new();

    let handshake_event = json!({
        "handshake_id": Uuid::new_v4(),

        "evaluation_id": evaluation_uuid,

        "client_id": client_id,

        "handshake_duration_ms": handshake_duration_ms,

        "kx_group": kx_group,

        "cipher_suite": cipher_suite,

        "success": true
    });

    if let Err(error) = http_client
        .post(format!("{}/eval/handshake-event", dashboard_url))
        .json(&handshake_event)
        .send()
        .await
    {
        eprintln!("Error sending handshake metric: {}", error);
    }

    let (metrics_stream, metrics_handle) = MetricsStream::new(Box::pin(tls_stream));

    let application_result = application(Box::pin(metrics_stream)).await;

    let bytes_transferred = metrics_handle.bytes_transferred();

    /*
     * The transfer metric must be sent before notifying
     * the Dashboard that this client has finished.
     *
     * Otherwise the last client may trigger evaluation
     * cleanup before its transfer metric is stored.
     */
    if bytes_transferred > 0 {
        let duration_ms = metrics_handle.duration_ms();

        let throughput_mbps = metrics_handle.throughput_mbps();

        let transfer_event = json!({
            "event_type": "transfer-completed",

            "execution_id": Uuid::new_v4().to_string(),

            "transfer_id": Uuid::new_v4(),

            "evaluation_id": evaluation_uuid,

            "client_id": client_id,

            "operation": "application",

            "file_size": bytes_transferred,

            "file_size_unit": "B",

            "num_files": 1,

            "bytes_transferred": bytes_transferred,

            "duration_ms": duration_ms,

            "throughput_mbps": throughput_mbps,

            "crypto_mode": crypto_config.crypto_mode.to_string(),

            "kx_group": kx_group,

            "cipher_suite": cipher_suite,

            "success": application_result.is_ok(),

            "error_type":
                if application_result.is_err() {
                    Some("application_error")
                } else {
                    None::<&str>
                }
        });

        if let Err(error) = http_client
            .post(format!("{}/eval/transfer-event", dashboard_url))
            .json(&transfer_event)
            .send()
            .await
        {
            eprintln!("Error sending transfer metric: {}", error);
        }
    }

    /*
     * Notify the Dashboard only after all metrics from
     * this client have been submitted.
     */
    let client_finished_event = json!({
        "evaluation_id": evaluation_uuid,

        "client_id": client_id
    });

    if let Err(error) = http_client
        .post(format!("{}/eval/client-finished", dashboard_url))
        .json(&client_finished_event)
        .send()
        .await
    {
        eprintln!("Error sending client completion event: {}", error);
    }

    application_result
}
