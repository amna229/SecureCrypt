use crate::user_application::BoxedApplicationStream;

use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;

/// Serves an established application stream using HTTP/1.1.
///
/// TLS is established by SecureCrypt before this function is called.
/// HTTP/1.1 only operates on the application stream and is therefore
/// independent from the concrete TLS implementation.
pub async fn serve_connection(
    application_stream: BoxedApplicationStream,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let io = TokioIo::new(application_stream);

    let router = super::routes::create_router();
    let service = TowerToHyperService::new(router);

    http1::Builder::new().serve_connection(io, service).await?;

    Ok(())
}
