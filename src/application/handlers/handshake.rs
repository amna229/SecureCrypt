//! TLS handshake handlers.
//!
//! This module stores metrics produced by completed TLS handshakes.

use axum::extract::{Json, Path, State};

use sqlx::PgPool;

use uuid::Uuid;

/// Metrics reported for a completed TLS handshake.
#[derive(Debug, serde::Deserialize)]
pub struct CompleteHandshakeRequest {
    pub evaluation_id: Uuid,
    pub client_id: i32,
    pub handshake_duration_ms: i64,
    pub kx_group: Option<String>,
    pub cipher_suite: Option<String>,
    pub success: bool,
}

/// Stores the metrics of a completed TLS handshake.
pub async fn complete_handshake(
    State(pool): State<PgPool>,
    Path(handshake_id): Path<Uuid>,
    Json(metrics): Json<CompleteHandshakeRequest>,
) -> Result<axum::http::StatusCode, axum::http::StatusCode> {
    sqlx::query(
        "INSERT INTO handshake (
            handshake_id,
            evaluation_id,
            client_id,
            handshake_duration_ms,
            kx_group,
            cipher_suite,
            success
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(handshake_id)
    .bind(metrics.evaluation_id)
    .bind(metrics.client_id)
    .bind(metrics.handshake_duration_ms)
    .bind(metrics.kx_group)
    .bind(metrics.cipher_suite)
    .bind(metrics.success)
    .execute(&pool)
    .await
    .map_err(|error| {
        eprintln!("Error saving handshake {}: {}", handshake_id, error);

        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    println!(
        "Handshake {} saved for client {}",
        handshake_id, metrics.client_id
    );

    Ok(axum::http::StatusCode::OK)
}
