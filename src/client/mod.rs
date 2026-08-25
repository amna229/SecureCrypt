use crate::client::config::TransferConfig;
use crate::client::metrics::CompleteTransferRequest;
use crate::client::transport::{
    complete_handshake, complete_transfer, create_empty_body, create_http_client,
    create_http_connection, create_upload_body, establish_tls_connection,
};
use crate::crypto::algorithm_provider::AlgorithmProvider;
use http_body_util::BodyExt;
use hyper::Request;
use std::error::Error;
use std::time::{Duration, Instant};
use uuid::Uuid;

pub mod config;
pub mod file_generator;
pub mod metrics;
pub mod transport;

type BoxError = Box<dyn Error + Send + Sync>;

/// Executes a complete client-side evaluation.
///
/// The client establishes a TLS connection using the selected
/// cryptographic provider, records the handshake metrics, waits
/// for a transfer configuration and executes the requested
/// upload or download operation.
pub async fn run_client(
    provider: Box<dyn AlgorithmProvider>,
    addr: &str,
    domain: &str,
    selected_cipher_suites: &[String],
    selected_kx_groups: &[String],
    config_url: &str,
    evaluation_started_at: &str,
    client_id: u32,
    evaluation_id: Uuid,
) -> Result<(), BoxError> {
    let crypto_mode = provider.name().to_string();

    let http_client = create_http_client()?;

    let handshake_id = Uuid::new_v4();

    let tls_connection_result = establish_tls_connection(
        provider,
        addr,
        domain,
        selected_cipher_suites,
        selected_kx_groups,
    )
    .await;

    let (tls_stream, handshake_duration_ms, kx_group, cipher_suite) = match tls_connection_result {
        Ok(connection) => connection,

        Err((error, handshake_duration_ms)) => {
            let handshake_metrics = crate::client::metrics::CompleteHandshakeRequest {
                evaluation_id,
                client_id: client_id as i32,
                handshake_duration_ms,
                kx_group: None,
                cipher_suite: None,
                success: false,
            };

            if let Err(report_error) =
                complete_handshake(&http_client, handshake_id, handshake_metrics).await
            {
                eprintln!(
                    "Error reporting failed handshake {}: {}",
                    handshake_id, report_error
                );
            }

            return Err(error);
        }
    };

    println!(
        "Client {}: TLS connection established with {}",
        client_id, addr
    );

    println!(
        "TLS handshake: {} ms | KX: {:?} | Cipher suite: {:?}",
        handshake_duration_ms, kx_group, cipher_suite
    );

    let handshake_metrics = crate::client::metrics::CompleteHandshakeRequest {
        evaluation_id,
        client_id: client_id as i32,
        handshake_duration_ms,
        kx_group: kx_group.clone(),
        cipher_suite: cipher_suite.clone(),
        success: true,
    };

    complete_handshake(&http_client, handshake_id, handshake_metrics).await?;

    let mut sender = create_http_connection(tls_stream).await?;

    let mut last_created_at = evaluation_started_at.parse::<i64>()?;

    loop {
        let latest_url = format!("{}?after={}", config_url, last_created_at);

        let response = http_client.get(&latest_url).send().await?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            tokio::time::sleep(Duration::from_secs(1)).await;

            continue;
        }

        let transfer_config = response
            .error_for_status()?
            .json::<TransferConfig>()
            .await?;

        println!(
            "Client {}: new transfer configuration: {:#?}",
            client_id, transfer_config
        );

        let execution_id = format!("{}_{:02}", transfer_config.transfer_id, client_id);

        let transfer_started_at = Instant::now();

        let transfer_result = match transfer_config.operation.as_str() {
            "upload" => execute_upload(&mut sender, &transfer_config, client_id).await,

            "download" => execute_download(&mut sender, &transfer_config, client_id).await,

            operation => Err(format!("Unknown operation: {}", operation).into()),
        };

        let duration = transfer_started_at.elapsed();

        match transfer_result {
            Ok(bytes_transferred) => {
                let duration_ms = duration.as_millis() as i64;

                let duration_seconds = duration.as_secs_f64();

                let throughput_mbps = if duration_seconds > 0.0 {
                    (bytes_transferred as f64 * 8.0) / duration_seconds / 1_000_000.0
                } else {
                    0.0
                };

                let metrics = CompleteTransferRequest {
                    duration_ms,
                    bytes_transferred: bytes_transferred as i64,
                    throughput_mbps,
                    crypto_mode: crypto_mode.clone(),
                    kx_group: kx_group.clone().unwrap_or_else(|| "unknown".to_string()),
                    cipher_suite: cipher_suite
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                    success: true,
                    error_type: None,
                };

                complete_transfer(&http_client, &execution_id, metrics).await?;

                println!(
                    "Client {}: transfer {} completed: {} bytes in {} ms ({:.2} Mbps)",
                    client_id, execution_id, bytes_transferred, duration_ms, throughput_mbps
                );
            }

            Err(error) => {
                let metrics = CompleteTransferRequest {
                    duration_ms: duration.as_millis() as i64,
                    bytes_transferred: 0,
                    throughput_mbps: 0.0,
                    crypto_mode: crypto_mode.clone(),
                    kx_group: kx_group.clone().unwrap_or_else(|| "unknown".to_string()),
                    cipher_suite: cipher_suite
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string()),
                    success: false,
                    error_type: Some(error.to_string()),
                };

                complete_transfer(&http_client, &execution_id, metrics).await?;

                eprintln!(
                    "Client {}: transfer {} failed: {}",
                    client_id, execution_id, error
                );
            }
        }

        println!("Evaluation ID: {}", evaluation_id);

        last_created_at = transfer_config.created_at;
    }
}

/// Executes an upload transfer.
///
/// The file is divided into chunks and sent through
/// the already established HTTP/1.1 connection.
async fn execute_upload(
    sender: &mut hyper::client::conn::http1::SendRequest<
        http_body_util::combinators::BoxBody<bytes::Bytes, BoxError>,
    >,
    transfer_config: &TransferConfig,
    client_id: u32,
) -> Result<u64, BoxError> {
    let mut bytes_transferred = 0u64;

    for file_id in 1..=transfer_config.num_files {
        let total_size = transfer_config.file_size_bytes;

        let body = create_upload_body(total_size)?;

        let request = Request::post("/upload")
            .header("Content-Type", "application/octet-stream")
            .header("Content-Length", total_size)
            .body(body)?;

        sender.ready().await?;

        println!("Client {}: HTTP connection ready for upload", client_id);

        let response = sender.send_request(request).await?;

        let status = response.status();

        response.into_body().collect().await?;

        if !status.is_success() {
            return Err(format!("Upload failed: {}", status).into());
        }

        bytes_transferred += total_size;

        println!(
            "Client {}: uploaded file {}/{}",
            client_id, file_id, transfer_config.num_files
        );
    }

    Ok(bytes_transferred)
}

/// Executes a download transfer.
///
/// The client requests each file from the server and
/// verifies that the expected number of bytes was received.
async fn execute_download(
    sender: &mut hyper::client::conn::http1::SendRequest<
        http_body_util::combinators::BoxBody<bytes::Bytes, BoxError>,
    >,
    transfer_config: &TransferConfig,
    client_id: u32,
) -> Result<u64, BoxError> {
    let mut bytes_transferred = 0u64;

    for file_id in 1..=transfer_config.num_files {
        let body = create_empty_body();

        let request = Request::get(format!(
            "/download?size={}",
            transfer_config.file_size_bytes
        ))
        .body(body)?;

        sender.ready().await?;

        println!("Client {}: HTTP connection ready for download", client_id);

        let response = sender.send_request(request).await?;

        let status = response.status();

        if !status.is_success() {
            response.into_body().collect().await?;

            return Err(format!("Download failed: {}", status).into());
        }

        let expected_size = transfer_config.file_size_bytes;

        let mut received_bytes = 0u64;

        let mut body = response.into_body();

        while let Some(frame_result) = body.frame().await {
            let frame = frame_result?;

            if let Some(data) = frame.data_ref() {
                received_bytes += data.len() as u64;
            }
        }

        if received_bytes != expected_size {
            return Err(format!(
                "Expected {} bytes, received {}",
                expected_size, received_bytes
            )
            .into());
        }

        bytes_transferred += received_bytes;

        println!(
            "Client {}: downloaded file {}/{}: {} bytes",
            client_id, file_id, transfer_config.num_files, received_bytes
        );
    }

    Ok(bytes_transferred)
}
