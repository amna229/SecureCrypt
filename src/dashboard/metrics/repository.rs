//! Evaluation repository.
//!
//! This module provides database operations for creating and
//! completing evaluations.

use chrono::{DateTime, Utc};

use sqlx::PgPool;

use uuid::Uuid;

/// Provides database operations for evaluations.
pub struct EvaluationRepository {
    pool: PgPool,
}

impl EvaluationRepository {
    /// Creates a new evaluation repository using the provided database pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Creates a new evaluation record in the database.
    ///
    /// New evaluations are initially stored with the `running` status.
    pub async fn create(
        &self,
        id: Uuid,
        crypto_mode: &str,
        num_clients: i32,
        started_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO evaluation (
                id,
                crypto_mode,
                num_clients,
                started_at,
                status
            )
            VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(crypto_mode)
        .bind(num_clients)
        .bind(started_at)
        .bind("running")
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Retrieves the completion timestamp of the latest finished transfer
    /// associated with the given evaluation.
    pub async fn get_last_transfer_finished_at(
        &self,
        evaluation_id: Uuid,
    ) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
        sqlx::query_scalar(
            "SELECT MAX(finished_at)
            FROM transfer
            WHERE evaluation_id = $1
            AND finished_at IS NOT NULL",
        )
        .bind(evaluation_id)
        .fetch_one(&self.pool)
        .await
    }

    /// Marks an evaluation as finished.
    ///
    /// The evaluation completion timestamp is obtained from the latest
    /// completed transfer associated with the evaluation.
    pub async fn finish(&self, id: Uuid, status: &str) -> Result<(), sqlx::Error> {
        let finished_at = self.get_last_transfer_finished_at(id).await?;

        sqlx::query(
            "UPDATE evaluation
            SET finished_at = $1,
                status = $2
            WHERE id = $3",
        )
        .bind(finished_at)
        .bind(status)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
