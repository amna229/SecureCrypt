//! Resource metrics repository.
//!
//! This module provides database operations for storing CPU and
//! memory usage metrics collected from evaluation containers.

use chrono::{DateTime, Utc};

use sqlx::PgPool;

use uuid::Uuid;

/// Provides database operations for container resource metrics.
pub struct ResourceRepository {
    pool: PgPool,
}

impl ResourceRepository {
    /// Creates a new resource repository using the provided database pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Stores resource usage metrics for a container.
    ///
    /// The recorded values include average and peak CPU usage together
    /// with peak memory consumption.
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
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
