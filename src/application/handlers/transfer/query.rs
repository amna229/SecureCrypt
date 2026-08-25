//! Transfer query handlers.
//!
//! This module retrieves transfer configurations from the database.

use axum::extract::{Json, Path, Query, State};

use sqlx::PgPool;

use uuid::Uuid;

/// Transfer configuration returned to a client.
#[derive(Debug, serde::Serialize)]
pub struct TransferConfigResponse {
    pub operation: String,
    pub file_size_bytes: u64,
    pub num_files: u32,
}

/// Latest transfer configuration returned to a client.
#[derive(Debug, serde::Serialize)]
pub struct LatestTransferResponse {
    pub transfer_id: Uuid,
    pub operation: String,
    pub file_size_bytes: u64,
    pub num_files: u32,
    pub created_at: i64,
}

/// Query parameter used to retrieve a transfer created
/// after a given timestamp.
#[derive(Debug, serde::Deserialize)]
pub struct LatestTransferQuery {
    pub after: i64,
}

/// Retrieves a transfer configuration by transfer identifier.
pub async fn get_transfer(
    State(pool): State<PgPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<TransferConfigResponse>, axum::http::StatusCode> {
    let row = sqlx::query_as::<_, (String, i64, String, i32)>(
        "SELECT
                operation,
                file_size,
                file_size_unit,
                num_files
             FROM transfer
             WHERE transfer_id = $1
             LIMIT 1",
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
                    return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let file_size_bytes = (file_size as u64)
                .checked_mul(multiplier)
                .ok_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(TransferConfigResponse {
                operation,
                file_size_bytes,
                num_files: num_files as u32,
            }))
        }

        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

/// Retrieves the first transfer created after the specified timestamp.
pub async fn get_latest_transfer(
    State(pool): State<PgPool>,
    Query(query): Query<LatestTransferQuery>,
) -> Result<Json<LatestTransferResponse>, axum::http::StatusCode> {
    let row = sqlx::query_as::<_, (Uuid, String, i64, String, i32, i64)>(
        "SELECT
                transfer_id,
                operation,
                file_size,
                file_size_unit,
                num_files,
                (
                    EXTRACT(
                        EPOCH FROM created_at
                    ) * 1000
                )::BIGINT
             FROM transfer
             WHERE (
                    EXTRACT(
                        EPOCH FROM created_at
                    ) * 1000
                  )::BIGINT > $1
             ORDER BY
                created_at ASC,
                transfer_id ASC,
                client_id ASC
             LIMIT 1",
    )
    .bind(query.after)
    .fetch_optional(&pool)
    .await
    .map_err(|error| {
        eprintln!("Error getting latest transfer: {}", error);

        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some((transfer_id, operation, file_size, file_size_unit, num_files, created_at)) => {
            let multiplier = match file_size_unit.as_str() {
                "KB" => 1024u64,
                "MB" => 1024u64 * 1024,
                "GB" => 1024u64 * 1024 * 1024,
                _ => {
                    return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
                }
            };

            let file_size_bytes = (file_size as u64)
                .checked_mul(multiplier)
                .ok_or(axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

            Ok(Json(LatestTransferResponse {
                transfer_id,
                operation,
                file_size_bytes,
                num_files: num_files as u32,
                created_at,
            }))
        }

        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}
