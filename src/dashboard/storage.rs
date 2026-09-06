//! Dashboard storage backends.
//!
//! The dashboard can use PostgreSQL for persistent evaluations or an
//! in-memory backend for standalone executions where no database is
//! available. The in-memory backend is intentionally process-local and
//! contains no persistent data.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MemoryEvaluation {
    pub id: Uuid,
    pub crypto_mode: String,
    pub num_clients: i32,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct MemoryHandshake {
    pub handshake_id: Uuid,
    pub evaluation_id: Uuid,
    pub client_id: i32,
    pub handshake_duration_ms: Option<i64>,
    pub kx_group: Option<String>,
    pub cipher_suite: Option<String>,
    pub success: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct MemoryTransfer {
    pub execution_id: String,
    pub transfer_id: Uuid,
    pub evaluation_id: Uuid,
    pub client_id: i32,
    pub operation: String,
    pub file_size: i64,
    pub file_size_unit: String,
    pub num_files: i32,
    pub bytes_transferred: Option<i64>,
    pub duration_ms: Option<i64>,
    pub throughput_mbps: Option<f64>,
    pub crypto_mode: Option<String>,
    pub kx_group: Option<String>,
    pub cipher_suite: Option<String>,
    pub success: Option<bool>,
    pub error_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct MemoryResource {
    pub resource_id: Uuid,
    pub evaluation_id: Uuid,
    pub role: String,
    pub client_id: Option<i32>,
    pub timestamp: DateTime<Utc>,
    pub cpu_avg_percent: f64,
    pub cpu_peak_percent: f64,
    pub memory_peak_bytes: i64,
}

#[derive(Debug, Default)]
pub struct MemoryDatabase {
    pub evaluations: Vec<MemoryEvaluation>,
    pub handshakes: Vec<MemoryHandshake>,
    pub transfers: Vec<MemoryTransfer>,
    pub resources: Vec<MemoryResource>,
}

#[derive(Clone)]
pub enum DashboardStorage {
    Postgres(PgPool),
    Memory(Arc<Mutex<MemoryDatabase>>),
}

impl DashboardStorage {
    pub fn memory() -> Self {
        Self::Memory(Arc::new(Mutex::new(MemoryDatabase::default())))
    }

    pub fn postgres(pool: PgPool) -> Self {
        Self::Postgres(pool)
    }

    pub fn is_memory(&self) -> bool {
        matches!(self, Self::Memory(_))
    }

    pub fn postgres_pool(&self) -> Option<PgPool> {
        match self {
            Self::Postgres(pool) => Some(pool.clone()),
            Self::Memory(_) => None,
        }
    }
}