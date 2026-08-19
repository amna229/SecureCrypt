use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use hyper_util::service::TowerToHyperService;

pub async fn serve_connection(
    tls_stream: tokio_rustls::server::TlsStream<tokio::net::TcpStream>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let io = TokioIo::new(tls_stream);

    let router = super::routes::create_router();

    let service = TowerToHyperService::new(router);

    http1::Builder::new().serve_connection(io, service).await?;

    Ok(())
}
