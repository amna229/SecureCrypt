//! Evaluation data model.
//!
//! This module defines the data structure used to represent an
//! evaluation stored in the database.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Represents an evaluation performed by the dashboard.
///
/// The structure stores the cryptographic mode used, the number of
/// clients involved, the evaluation timestamps, and its final status.
#[derive(Debug)]
pub struct Evaluation {
    pub id: Uuid,
    pub crypto_mode: String,
    pub num_clients: i32,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub status: String,
}
