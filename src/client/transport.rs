use crate::client::metrics::CompleteHandshakeRequest;
use crate::crypto::algorithm_provider::AlgorithmProvider;

use bytes::Bytes;
use http_body_util::{BodyExt, Full, StreamBody, combinators::BoxBody};
use hyper::body::Frame;
use hyper_util::rt::TokioIo;
use reqwest::Client;
use rustls::pki_types::ServerName;
use std::convert::Infallible;
use std::error::Error;
use std::sync::Arc;
use std::time::Instant;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use uuid::Uuid;

type BoxError = Box<dyn Error + Send + Sync>;

/// Creates the HTTP client used to communicate with
/// the application service and report evaluation metrics.
pub fn create_http_client() -> Result<Client, BoxError> {
    let ca_cert = std::fs::read("simplified-pki/rootCA/rootCA.crt")?;

    let cert = reqwest::Certificate::from_pem(&ca_cert)?;

    let client = Client::builder().add_root_certificate(cert).build()?;

    Ok(client)
}

/// Establishes a TLS connection with the evaluation server.
///
/// The function measures the TLS handshake duration and
/// extracts the negotiated key exchange group and cipher suite.
///
/// If the handshake fails, the error and the measured
/// handshake duration are returned to the caller.
pub async fn establish_tls_connection(
    provider: Box<dyn AlgorithmProvider>,
    addr: &str,
    domain: &str,
    selected_cipher_suites: &[String],
    selected_kx_groups: &[String],
) -> Result<
    (
        tokio_rustls::client::TlsStream<TcpStream>,
        i64,
        Option<String>,
        Option<String>,
    ),
    (BoxError, i64),
> {
    let config = provider
        .build_client_config(selected_cipher_suites, selected_kx_groups)
        .map_err(|error| (error.into(), 0))?;

    let tls_connector = TlsConnector::from(Arc::new(config));

    let tcp_stream = connect_to_server_tcp(addr)
        .await
        .map_err(|error| (error, 0))?;

    let server_name =
        ServerName::try_from(domain.to_string()).map_err(|error| (error.into(), 0))?;

    let handshake_started_at = Instant::now();

    let tls_stream_result = tls_connector.connect(server_name, tcp_stream).await;

    let handshake_duration_ms = handshake_started_at.elapsed().as_millis() as i64;

    let tls_stream = match tls_stream_result {
        Ok(tls_stream) => tls_stream,

        Err(error) => {
            return Err((error.into(), handshake_duration_ms));
        }
    };

    let (_, tls_connection) = tls_stream.get_ref();

    let cipher_suite = tls_connection
        .negotiated_cipher_suite()
        .map(|suite| format!("{:?}", suite.suite()));

    let kx_group = tls_connection
        .negotiated_key_exchange_group()
        .map(|group| format!("{:?}", group.name()));

    Ok((tls_stream, handshake_duration_ms, kx_group, cipher_suite))
}

/// Reports a completed TLS handshake to the application service.
pub async fn complete_handshake(
    http_client: &Client,
    handshake_id: Uuid,
    metrics: CompleteHandshakeRequest,
) -> Result<(), BoxError> {
    let url = format!(
        "https://securecrypt-application:8443/handshake/{}/complete",
        handshake_id
    );

    let response = http_client.post(&url).json(&metrics).send().await?;

    if !response.status().is_success() {
        return Err(format!(
            "Error saving handshake {}: {}",
            handshake_id,
            response.status()
        )
        .into());
    }

    Ok(())
}

/// Reports completed transfer metrics to the application service.
pub async fn complete_transfer(
    http_client: &Client,
    execution_id: &str,
    metrics: crate::client::metrics::CompleteTransferRequest,
) -> Result<(), BoxError> {
    let url = format!(
        "https://securecrypt-application:8443/transfer/{}/complete",
        execution_id
    );

    let response = http_client.post(&url).json(&metrics).send().await?;

    if !response.status().is_success() {
        return Err(format!(
            "Error completing transfer {}: {}",
            execution_id,
            response.status()
        )
        .into());
    }

    Ok(())
}

/// Opens a TCP connection with the server.
async fn connect_to_server_tcp(addr: &str) -> Result<TcpStream, BoxError> {
    let stream = TcpStream::connect(addr).await?;

    Ok(stream)
}

/// Creates an HTTP/1.1 connection over an established TLS stream.
pub async fn create_http_connection(
    tls_stream: tokio_rustls::client::TlsStream<TcpStream>,
) -> Result<hyper::client::conn::http1::SendRequest<BoxBody<Bytes, BoxError>>, BoxError> {
    let io = TokioIo::new(tls_stream);

    let (sender, connection) = hyper::client::conn::http1::handshake(io).await?;

    tokio::spawn(async move {
        match connection.await {
            Ok(()) => {
                eprintln!("HTTP connection closed normally");
            }

            Err(error) => {
                eprintln!("HTTP connection driver failed: {}", error);
            }
        }
    });

    Ok(sender)
}

/// Creates an empty HTTP request body.
pub fn create_empty_body() -> BoxBody<Bytes, BoxError> {
    Full::new(Bytes::new()).map_err(boxed_infallible).boxed()
}

/// Creates a streaming HTTP request body for an upload.
///
/// The data is generated in chunks instead of allocating
/// the complete file in memory at once.
pub fn create_upload_body(total_size: u64) -> Result<BoxBody<Bytes, BoxError>, BoxError> {
    const CHUNK_SIZE: u64 = 1024 * 1024;

    let chunks = (total_size + CHUNK_SIZE - 1) / CHUNK_SIZE;

    let body_stream = futures_util::stream::iter((0..chunks).map(move |chunk_id| {
        let offset = chunk_id * CHUNK_SIZE;

        let remaining = total_size - offset;

        let chunk_size = remaining.min(CHUNK_SIZE);

        let data = crate::client::file_generator::generate_file(chunk_size);

        Ok::<Frame<Bytes>, Infallible>(Frame::data(Bytes::from(data)))
    }));

    let body = StreamBody::new(body_stream)
        .map_err(boxed_infallible)
        .boxed();

    Ok(body)
}

/// Converts an infallible error into the common boxed error type.
fn boxed_infallible(error: Infallible) -> BoxError {
    match error {}
}
