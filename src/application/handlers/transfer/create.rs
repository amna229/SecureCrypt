//! Transfer creation handler.
//!
//! This module creates transfer configurations for the
//! clients participating in an evaluation.

use crate::application::{
    config::ApplicationConfig, db::repository::ApplicationRepository,
    handlers::notification::notify_dashboard,
};

use axum::extract::{Json, State};

use sqlx::PgPool;

use uuid::Uuid;

/// Response returned after creating a transfer.
#[derive(Debug, serde::Serialize)]
pub struct TransferResponse {
    pub transfer_id: Uuid,
}

/// Creates a new transfer configuration.
pub async fn start_app(
    State(pool): State<PgPool>,
    Json(config): Json<ApplicationConfig>,
) -> Json<TransferResponse> {
    println!("{:#?}", config);

    let repository = ApplicationRepository::new(pool);

    let transfer_id = repository
        .save(&config)
        .await
        .expect("Failed to save transfer");

    println!("Transfer saved: {}", transfer_id);

    notify_dashboard(
        "transfer-started",
        config.evaluation_id,
        &format!("{}_01", transfer_id),
        1,
        &config.operation,
        true,
    )
    .await;

    Json(TransferResponse { transfer_id })
}
