use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;

/// Serves an established TLS connection using HTTP/1.1.
///
/// The TLS handshake must have been completed before calling this function.
/// The established TLS stream is adapted to Hyper's I/O interface and the
/// application's Axum router is used to handle incoming HTTP requests.
pub async fn serve_connection(
    tls_stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let io = TokioIo::new(tls_stream);

    let router = super::routes::create_router();
    let service = TowerToHyperService::new(router);

    http1::Builder::new().serve_connection(io, service).await?;

    Ok(())
}
