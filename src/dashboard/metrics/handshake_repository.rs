//! TLS handshake metrics repository.

use crate::dashboard::database::storage::{DashboardStorage, MemoryHandshake};
use uuid::Uuid;

/// Provides storage operations for TLS handshake metrics.
pub struct HandshakeRepository {
    storage: DashboardStorage,
}

impl HandshakeRepository {
    /// Creates a repository using the selected dashboard storage backend.
    pub fn new(storage: DashboardStorage) -> Self {
        Self { storage }
    }

    /// Stores a TLS handshake measurement.
    pub async fn save(
        &self,
        handshake_id: Uuid,
        evaluation_id: Uuid,
        client_id: i32,
        handshake_duration_ms: Option<i64>,
        kx_group: Option<&str>,
        cipher_suite: Option<&str>,
        success: bool,
    ) -> Result<(), sqlx::Error> {
        match &self.storage {
            DashboardStorage::Postgres(pool) => {
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
                .bind(evaluation_id)
                .bind(client_id)
                .bind(handshake_duration_ms)
                .bind(kx_group)
                .bind(cipher_suite)
                .bind(success)
                .execute(pool)
                .await?;

                Ok(())
            }

            DashboardStorage::Memory(database) => {
                let mut database = database.lock().await;

                database.handshakes.push(MemoryHandshake {
                    handshake_id,
                    evaluation_id,
                    client_id,
                    handshake_duration_ms,
                    kx_group: kx_group.map(str::to_string),
                    cipher_suite: cipher_suite.map(str::to_string),
                    success,
                    created_at: chrono::Utc::now(),
                });

                Ok(())
            }
        }
    }
}
