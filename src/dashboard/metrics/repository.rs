//! Evaluation repository.
//!
//! This module stores and completes evaluation records using either
//! PostgreSQL or temporary in-memory storage.

use crate::dashboard::database::storage::{DashboardStorage, MemoryEvaluation};

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Provides storage operations for evaluations.
pub struct EvaluationRepository {
    storage: DashboardStorage,
}

impl EvaluationRepository {
    /// Creates an evaluation repository using the selected storage backend.
    pub fn new(storage: DashboardStorage) -> Self {
        Self { storage }
    }

    /// Creates a new running evaluation.
    pub async fn create(
        &self,
        id: Uuid,
        crypto_mode: &str,
        num_clients: i32,
        started_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        match &self.storage {
            DashboardStorage::Postgres(pool) => {
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
                .execute(pool)
                .await?;

                Ok(())
            }

            DashboardStorage::Memory(database) => {
                let mut database = database.lock().await;

                database.evaluations.push(MemoryEvaluation {
                    id,
                    crypto_mode: crypto_mode.to_string(),
                    num_clients,
                    started_at,
                    finished_at: None,
                    status: "running".to_string(),
                });

                Ok(())
            }
        }
    }

    /// Retrieves the latest completed transfer timestamp associated
    /// with an evaluation.
    pub async fn get_last_transfer_finished_at(
        &self,
        evaluation_id: Uuid,
    ) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
        match &self.storage {
            DashboardStorage::Postgres(pool) => {
                sqlx::query_scalar(
                    "SELECT MAX(finished_at)
                     FROM transfer
                     WHERE evaluation_id = $1
                     AND finished_at IS NOT NULL",
                )
                .bind(evaluation_id)
                .fetch_one(pool)
                .await
            }

            DashboardStorage::Memory(database) => {
                let database = database.lock().await;

                Ok(database
                    .transfers
                    .iter()
                    .filter(|transfer| transfer.evaluation_id == evaluation_id)
                    .filter_map(|transfer| transfer.finished_at)
                    .max())
            }
        }
    }

    /// Marks an evaluation as completed.
    pub async fn finish(&self, id: Uuid, status: &str) -> Result<(), sqlx::Error> {
        let finished_at = self.get_last_transfer_finished_at(id).await?;

        match &self.storage {
            DashboardStorage::Postgres(pool) => {
                sqlx::query(
                    "UPDATE evaluation
                     SET finished_at = $1,
                         status = $2
                     WHERE id = $3",
                )
                .bind(finished_at)
                .bind(status)
                .bind(id)
                .execute(pool)
                .await?;

                Ok(())
            }

            DashboardStorage::Memory(database) => {
                let mut database = database.lock().await;

                if let Some(evaluation) = database
                    .evaluations
                    .iter_mut()
                    .find(|evaluation| evaluation.id == id)
                {
                    evaluation.finished_at = finished_at;
                    evaluation.status = status.to_string();
                }

                Ok(())
            }
        }
    }
}
