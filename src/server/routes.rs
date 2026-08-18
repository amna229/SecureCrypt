use axum::{
    body::Body,
    extract::{
        DefaultBodyLimit,
        Query,
    },
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::{get, post,},
    Router,
};
use http_body_util::BodyExt;
use serde::Deserialize;
use bytes::Bytes;
use futures_util::stream;




#[derive(Debug, Deserialize)]
struct DownloadQuery {
    size: u64,
}



pub fn create_router() -> Router {

    Router::new()
        .route("/upload", post(upload))
        .route("/download", get(download))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024 * 1024)
        )

}



async fn upload(mut body: Body) -> impl IntoResponse {

    let mut total_bytes: u64 = 0;


    while let Some(frame_result) = body.frame().await{

        match frame_result {

            Ok(frame) => {

                if let Ok(data) = frame.into_data()
                {

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



async fn download(Query(query): Query<DownloadQuery>) -> Response {

    const CHUNK_SIZE: u64 = 1024 * 1024;

    let total_size = query.size;

    let chunks = (total_size + CHUNK_SIZE - 1) / CHUNK_SIZE;

    let body_stream =
        stream::iter(
            (0..chunks).map(
                move |chunk_id| {

                    let offset = chunk_id * CHUNK_SIZE;

                    let remaining = total_size - offset;

                    let chunk_size = remaining.min(CHUNK_SIZE);

                    Ok::<Bytes, std::convert::Infallible>(
                        Bytes::from(
                            vec![
                                0u8;
                                chunk_size as usize
                            ]
                        )
                    )
                }
            )
        );

    let body = Body::from_stream(body_stream);

    Response::builder()
        .status(StatusCode::OK)
        .header(
            "Content-Type",
            "application/octet-stream"
        )
        .body(body)
        .unwrap()
}