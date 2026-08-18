use axum::{
    extract::{Json, Path, Query, State},
    response::Html,
};
use crate::application::config::ApplicationConfig;
use sqlx::PgPool;
use uuid::Uuid;




#[derive(serde::Serialize)]
pub struct TransferResponse {
    pub transfer_id: Uuid
}




#[derive(Debug, serde::Serialize)]
pub struct TransferConfigResponse {
    pub operation: String,
    pub file_size_bytes: u64,
    pub num_files: u32,
}




#[derive(Debug, serde::Serialize)]
pub struct LatestTransferResponse {
    pub transfer_id: Uuid,
    pub operation: String,
    pub file_size_bytes: u64,
    pub num_files: u32,
    pub created_at: i64
}




#[derive(Debug, serde::Deserialize)]
pub struct LatestTransferQuery {
    pub after: i64,
}




pub async fn my_app() -> Html<&'static str> {

    Html(include_str!("ui/templates/myApp.html"))

}




pub async fn start_app(
    State(pool): State<PgPool>,
    Json(config): Json<ApplicationConfig>
) -> Json<TransferResponse> {

    println!("{:#?}", config);

    let repository =
        crate::application::db::repository::ApplicationRepository::new(pool);

    let transfer_id = repository
        .save(&config)
        .await
        .expect("Failed to save transfer");

    println!("Transfer saved: {}", transfer_id);

    Json(TransferResponse {transfer_id})

}




pub async fn get_transfer(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>
) -> Result<Json<TransferConfigResponse>, axum::http::StatusCode> {

    let row = sqlx::query_as::<_, (String, i64, String, i32)>(
        "SELECT operation, file_size, file_size_unit, num_files
         FROM transfer
         WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    match row {
        Some((operation, file_size, file_size_unit, num_files)) => {

            let multiplier = match file_size_unit.as_str() {
                "KB" => 1024u64,
                "MB" => 1024u64 * 1024,
                "GB" => 1024u64 * 1024 * 1024,
                _ => {
                    return Err(
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR
                    );
                }
            };

            let file_size_bytes = (file_size as u64)
                .checked_mul(multiplier)
                .ok_or(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR
                )?;

            Ok(Json(TransferConfigResponse {
                operation,
                file_size_bytes,
                num_files: num_files as u32,
            }))
        }

        None => Err(axum::http::StatusCode::NOT_FOUND),
    }

}




pub async fn get_latest_transfer(
    State(pool): State<PgPool>,
    Query(query): Query<LatestTransferQuery>,
) -> Result<Json<LatestTransferResponse>, axum::http::StatusCode> {

    let row = sqlx::query_as::<_, (
        Uuid,
        String,
        i64,
        String,
        i32,
        i64
    )>(
        "SELECT id,
                operation,
                file_size,
                file_size_unit,
                num_files,
                (EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT
         FROM transfer
         WHERE (EXTRACT(EPOCH FROM created_at) * 1000)::BIGINT > $1
         ORDER BY created_at ASC, id ASC
         LIMIT 1"
    )
    .bind(query.after)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        eprintln!(
            "Error getting latest transfer: {}",
            e
        );

        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {

        Some((
            id,
            operation,
            file_size,
            file_size_unit,
            num_files,
            created_at
        )) => {

            let multiplier =
                match file_size_unit.as_str() {

                    "KB" => 1024u64,

                    "MB" =>
                        1024u64 * 1024,

                    "GB" =>
                        1024u64 * 1024 * 1024,

                    _ => {
                        return Err(
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR
                        );
                    }
                };


            let file_size_bytes =
                (file_size as u64)
                    .checked_mul(multiplier)
                    .ok_or(
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR
                    )?;


            Ok(Json(
                LatestTransferResponse {
                    transfer_id: id,
                    operation,
                    file_size_bytes,
                    num_files: num_files as u32,
                    created_at
                }
            ))
        }

        None => {
            Err(
                axum::http::StatusCode::NOT_FOUND
            )
        }
    }
}