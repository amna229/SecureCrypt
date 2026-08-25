//! Application database repository.
//!
//! This module provides database operations for creating transfer
//! configurations associated with an active evaluation.

use crate::application::config::ApplicationConfig;

use sqlx::PgPool;

use uuid::Uuid;

/// Provides database operations for the application.
pub struct ApplicationRepository {
    pool: PgPool,
}

impl ApplicationRepository {
    /// Creates a new application repository using the provided database pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new transfer configuration for the current evaluation.
    ///
    /// A transfer identifier is generated and one transfer execution
    /// record is created for each client participating in the evaluation.
    ///
    /// All transfer records are inserted within a single database
    /// transaction so that the configuration is created atomically.
    pub async fn save(&self, config: &ApplicationConfig) -> Result<Uuid, sqlx::Error> {
        let transfer_id = Uuid::new_v4();

        let (crypto_mode, num_clients): (String, i32) = sqlx::query_as(
            "SELECT crypto_mode, num_clients
             FROM evaluation
             WHERE id = $1",
        )
        .bind(config.evaluation_id)
        .fetch_one(&self.pool)
        .await?;

        let mut transaction = self.pool.begin().await?;

        for client_id in 1..=num_clients {
            let execution_id = format!("{}_{:02}", transfer_id, client_id);

            sqlx::query(
                "INSERT INTO transfer (
                    execution_id,
                    transfer_id,
                    evaluation_id,
                    client_id,
                    operation,
                    file_size,
                    file_size_unit,
                    num_files,
                    crypto_mode
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            )
            .bind(&execution_id)
            .bind(transfer_id)
            .bind(config.evaluation_id)
            .bind(client_id)
            .bind(&config.operation)
            .bind(config.file_size as i64)
            .bind(&config.file_size_unit)
            .bind(config.num_files as i32)
            .bind(&crypto_mode)
            .execute(&mut *transaction)
            .await?;
        }

        transaction.commit().await?;

        println!(
            "Transfer {} created for {} clients",
            transfer_id, num_clients
        );

        Ok(transfer_id)
    }
}
