use axum::{
    Router,
    body::Body,
    extract::{DefaultBodyLimit, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use bytes::Bytes;
use futures_util::stream;
use http_body_util::BodyExt;
use serde::Deserialize;

/// Query parameters accepted by the download endpoint.
#[derive(Debug, Deserialize)]
struct DownloadQuery {
    /// Number of bytes to generate and return to the client.
    size: u64,
}

/// Creates the HTTP router for the application.
///
/// The router exposes endpoints for uploading and downloading data.
/// A maximum request body size of 10 GiB is configured to support
/// large file transfers while still providing an explicit upper limit.
pub fn create_router() -> Router {
    Router::new()
        .route("/upload", post(upload))
        .route("/download", get(download))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024 * 1024))
}

/// Receives an uploaded data stream from the client.
///
/// The request body is processed incrementally rather than being loaded
/// completely into memory. The total number of bytes received is counted
/// and logged once the transfer has completed.
async fn upload(mut body: Body) -> impl IntoResponse {
    let mut total_bytes: u64 = 0;

    while let Some(frame_result) = body.frame().await {
        match frame_result {
            Ok(frame) => {
                if let Ok(data) = frame.into_data() {
                    total_bytes += data.len() as u64;
                }
            }

            Err(error) => {
                eprintln!("Error receiving upload body: {}", error);

                return StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    }

    println!("Server: received {} bytes", total_bytes);

    StatusCode::OK
}

/// Generates a data stream of the requested size for the client.
///
/// Data is generated in 1 MiB chunks to avoid allocating the complete
/// response body in memory at once. The generated content is intentionally
/// synthetic because the endpoint is used to measure transfer performance.
async fn download(Query(query): Query<DownloadQuery>) -> Response {
    const CHUNK_SIZE: u64 = 1024 * 1024;

    let total_size = query.size;

    let chunks = (total_size + CHUNK_SIZE - 1) / CHUNK_SIZE;

    let body_stream = stream::iter((0..chunks).map(move |chunk_id| {
        let offset = chunk_id * CHUNK_SIZE;
        let remaining = total_size - offset;
        let chunk_size = remaining.min(CHUNK_SIZE);

        Ok::<Bytes, std::convert::Infallible>(Bytes::from(vec![0u8; chunk_size as usize]))
    }));

    let body = Body::from_stream(body_stream);

    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "application/octet-stream")
        .body(body)
        .unwrap()
}
