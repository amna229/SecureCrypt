//! File transfer metrics repository.

use crate::dashboard::database::storage::{DashboardStorage, MemoryTransfer};

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Provides storage operations for file transfer metrics.
pub struct TransferRepository {
    storage: DashboardStorage,
}

impl TransferRepository {
    /// Creates a repository using the selected dashboard storage backend.
    pub fn new(storage: DashboardStorage) -> Self {
        Self { storage }
    }

    /// Stores a file transfer measurement.
    pub async fn save(
        &self,
        execution_id: &str,
        transfer_id: Uuid,
        evaluation_id: Uuid,
        client_id: i32,
        operation: &str,
        file_size: i64,
        file_size_unit: &str,
        num_files: i32,
        bytes_transferred: Option<i64>,
        duration_ms: Option<i64>,
        throughput_mbps: Option<f64>,
        crypto_mode: Option<&str>,
        kx_group: Option<&str>,
        cipher_suite: Option<&str>,
        success: Option<bool>,
        error_type: Option<&str>,
        created_at: DateTime<Utc>,
        finished_at: Option<DateTime<Utc>>,
    ) -> Result<(), sqlx::Error> {
        match &self.storage {
            DashboardStorage::Postgres(pool) => {
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
                        bytes_transferred,
                        duration_ms,
                        throughput_mbps,
                        crypto_mode,
                        kx_group,
                        cipher_suite,
                        success,
                        error_type,
                        created_at,
                        finished_at
                    )
                    VALUES (
                        $1, $2, $3, $4, $5, $6, $7, $8,
                        $9, $10, $11, $12, $13, $14, $15, $16,
                        $17, $18
                    )",
                )
                .bind(execution_id)
                .bind(transfer_id)
                .bind(evaluation_id)
                .bind(client_id)
                .bind(operation)
                .bind(file_size)
                .bind(file_size_unit)
                .bind(num_files)
                .bind(bytes_transferred)
                .bind(duration_ms)
                .bind(throughput_mbps)
                .bind(crypto_mode)
                .bind(kx_group)
                .bind(cipher_suite)
                .bind(success)
                .bind(error_type)
                .bind(created_at)
                .bind(finished_at)
                .execute(pool)
                .await?;

                Ok(())
            }

            DashboardStorage::Memory(database) => {
                let mut database = database.lock().await;

                database.transfers.push(MemoryTransfer {
                    execution_id: execution_id.to_string(),
                    transfer_id,
                    evaluation_id,
                    client_id,
                    operation: operation.to_string(),
                    file_size,
                    file_size_unit: file_size_unit.to_string(),
                    num_files,
                    bytes_transferred,
                    duration_ms,
                    throughput_mbps,
                    crypto_mode: crypto_mode.map(str::to_string),
                    kx_group: kx_group.map(str::to_string),
                    cipher_suite: cipher_suite.map(str::to_string),
                    success,
                    error_type: error_type.map(str::to_string),
                    created_at,
                    finished_at,
                });

                Ok(())
            }
        }
    }
}
