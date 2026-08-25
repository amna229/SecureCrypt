//! Evaluation results repository.
//!
//! This module defines the data structures used to represent aggregated
//! evaluation results and exposes the repository responsible for loading
//! and processing those results.

mod derived;
mod loader;
mod summaries;

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

/// Represents a summarized evaluation.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EvaluationSummary {
    #[serde(skip_serializing)]
    pub id: Uuid,
    pub crypto_mode: String,
    pub num_clients: i32,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: String,
}

/// Represents aggregated TLS handshake metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct HandshakeSummary {
    pub crypto_mode: String,
    pub kx_group: String,
    pub samples: i64,
    pub mean_ms: f64,
    pub median_ms: f64,
    pub stddev_ms: f64,
    pub min_ms: i64,
    pub max_ms: i64,
}

/// Represents aggregated transfer metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TransferSummary {
    pub crypto_mode: String,
    pub kx_group: String,
    pub samples: i64,
    pub mean_duration_ms: f64,
    pub mean_throughput_mbps: f64,
    pub total_bytes: i64,
}

/// Represents aggregated container resource metrics.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResourceSummary {
    pub crypto_mode: String,
    pub role: String,
    pub samples: i64,
    pub mean_cpu_avg_percent: f64,
    pub max_cpu_peak_percent: f64,
    pub mean_memory_peak_bytes: f64,
    pub max_memory_peak_bytes: i64,
}

/// Represents metrics derived from the comparison between
/// classical and post-quantum configurations.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DerivedMetrics {
    pub handshake_overhead_percent: Option<f64>,
    pub transfer_overhead_percent: Option<f64>,
    pub cpu_overhead_percent: Option<f64>,
    pub memory_overhead_percent: Option<f64>,
    pub throughput_change_percent: Option<f64>,
}

/// Represents a group of evaluations sharing the same
/// experimental configuration.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EvaluationGroup {
    pub num_clients: i32,
    pub operation: String,
    pub file_size: i64,
    pub file_size_unit: String,
    pub num_files: i32,
    pub evaluations: Vec<EvaluationSummary>,
    pub handshakes: Vec<HandshakeSummary>,
    pub transfers: Vec<TransferSummary>,
    pub resources: Vec<ResourceSummary>,
    pub derived: DerivedMetrics,
}

/// Represents all results returned for a requested date.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ResultsData {
    pub groups: Vec<EvaluationGroup>,
}

/// Identifies an experimental configuration.
///
/// Evaluations with the same values belong to the same result group.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct GroupKey {
    pub(crate) num_clients: i32,
    pub(crate) operation: String,
    pub(crate) file_size: i64,
    pub(crate) file_size_unit: String,
    pub(crate) num_files: i32,
}

/// Provides database access to evaluation results.
pub struct ResultsRepository {
    pub(crate) pool: PgPool,
}

impl ResultsRepository {
    /// Creates a new results repository using the provided database pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub(crate) use derived::calculate_derived;

pub(crate) use summaries::{
    build_handshake_summary, build_resource_summary, build_transfer_summary,
};
