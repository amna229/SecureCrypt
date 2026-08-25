//! HTTP/1.1 connection handling.
//!
//! This module adapts an established TLS connection to Hyper HTTP/1.1
//! and serves the application router over that connection.

use crate::application::routes;

use hyper::server::conn::http1;

use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;

/// Serves an HTTP/1.1 connection over an established TLS stream.
///
/// The application router is created with the shared database pool
/// and then served using Hyper.
pub async fn serve_connection(
    tls_stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
    pool: sqlx::PgPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let io = TokioIo::new(tls_stream);

    let router = routes::create_router(pool);

    let service = TowerToHyperService::new(router);

    http1::Builder::new().serve_connection(io, service).await?;

    Ok(())
}
