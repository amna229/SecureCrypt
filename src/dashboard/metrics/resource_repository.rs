//! Resource metrics repository.
//!
//! This module stores CPU and memory metrics using either PostgreSQL
//! or temporary in-memory storage.

use crate::dashboard::database::storage::{DashboardStorage, MemoryResource};

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Provides storage operations for resource metrics.
pub struct ResourceRepository {
    storage: DashboardStorage,
}

impl ResourceRepository {
    /// Creates a repository using the selected storage backend.
    pub fn new(storage: DashboardStorage) -> Self {
        Self { storage }
    }

    /// Stores CPU and memory measurements for an evaluation container.
    pub async fn save(
        &self,
        resource_id: Uuid,
        evaluation_id: Uuid,
        role: &str,
        client_id: Option<i32>,
        timestamp: DateTime<Utc>,
        cpu_avg_percent: f64,
        cpu_peak_percent: f64,
        memory_peak_bytes: i64,
    ) -> Result<(), sqlx::Error> {
        match &self.storage {
            DashboardStorage::Postgres(pool) => {
                sqlx::query(
                    "INSERT INTO resource (
                        resource_id,
                        evaluation_id,
                        role,
                        client_id,
                        timestamp,
                        cpu_avg_percent,
                        cpu_peak_percent,
                        memory_peak_bytes
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                )
                .bind(resource_id)
                .bind(evaluation_id)
                .bind(role)
                .bind(client_id)
                .bind(timestamp)
                .bind(cpu_avg_percent)
                .bind(cpu_peak_percent)
                .bind(memory_peak_bytes)
                .execute(pool)
                .await?;

                Ok(())
            }

            DashboardStorage::Memory(database) => {
                let mut database = database.lock().await;

                database.resources.push(MemoryResource {
                    resource_id,
                    evaluation_id,
                    role: role.to_string(),
                    client_id,
                    timestamp,
                    cpu_avg_percent,
                    cpu_peak_percent,
                    memory_peak_bytes,
                });

                Ok(())
            }
        }
    }
}
