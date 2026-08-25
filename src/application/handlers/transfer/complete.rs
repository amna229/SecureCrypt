//! Transfer completion handler.
//!
//! This module stores transfer performance metrics and
//! notifies the dashboard when a transfer finishes.

use crate::application::handlers::notification::notify_dashboard;

use axum::extract::{Json, Path, State};

use sqlx::PgPool;

use uuid::Uuid;

/// Metrics reported when a transfer execution finishes.
#[derive(Debug, serde::Deserialize)]
pub struct CompleteTransferRequest {
    pub duration_ms: i64,
    pub bytes_transferred: i64,
    pub throughput_mbps: f64,
    pub crypto_mode: String,
    pub kx_group: String,
    pub cipher_suite: String,
    pub success: bool,
    pub error_type: Option<String>,
}

/// Stores the completion metrics of a transfer execution.
pub async fn complete_transfer(
    State(pool): State<PgPool>,
    Path(execution_id): Path<String>,
    Json(metrics): Json<CompleteTransferRequest>,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    let transfer_info = sqlx::query_as::<_, (Uuid, i32, String)>(
        "SELECT
                evaluation_id,
                client_id,
                operation
             FROM transfer
             WHERE execution_id = $1",
    )
    .bind(&execution_id)
    .fetch_optional(&pool)
    .await
    .map_err(|error| {
        eprintln!("Error finding transfer {}: {}", execution_id, error);

        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let (evaluation_id, client_id, operation) = match transfer_info {
        Some(info) => info,

        None => {
            eprintln!("Transfer execution not found: {}", execution_id);

            return Err(axum::http::StatusCode::NOT_FOUND);
        }
    };

    let result = sqlx::query(
        "UPDATE transfer
             SET finished_at = NOW(),
                 duration_ms = $1,
                 bytes_transferred = $2,
                 throughput_mbps = $3,
                 crypto_mode = $4,
                 kx_group = $5,
                 cipher_suite = $6,
                 success = $7,
                 error_type = $8
             WHERE execution_id = $9",
    )
    .bind(metrics.duration_ms)
    .bind(metrics.bytes_transferred)
    .bind(metrics.throughput_mbps)
    .bind(&metrics.crypto_mode)
    .bind(&metrics.kx_group)
    .bind(&metrics.cipher_suite)
    .bind(metrics.success)
    .bind(&metrics.error_type)
    .bind(&execution_id)
    .execute(&pool)
    .await
    .map_err(|error| {
        eprintln!("Error completing transfer {}: {}", execution_id, error);

        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() != 1 {
        eprintln!("Transfer execution not found: {}", execution_id);

        return Err(axum::http::StatusCode::NOT_FOUND);
    }

    println!("Transfer execution {} completed successfully", execution_id);

    notify_dashboard(
        "transfer-completed",
        evaluation_id,
        &execution_id,
        client_id,
        &operation,
        metrics.success,
    )
    .await;

    Ok(axum::http::StatusCode::OK)
}
