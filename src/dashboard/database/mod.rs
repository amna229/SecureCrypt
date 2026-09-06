//! Dashboard database initialization.
//!
//! PostgreSQL is used when it is available. When no usable PostgreSQL
//! connection can be established, the dashboard automatically falls back
//! to an in-memory storage backend.

pub mod storage;

use sqlx::{PgPool, postgres::PgPoolOptions};

use self::storage::DashboardStorage;

/// Creates the storage backend used by the dashboard.
///
/// PostgreSQL is attempted when `DATABASE_URL` is defined. If the
/// environment variable is missing or the connection cannot be established,
/// an in-memory backend is returned instead.
///
/// The in-memory backend is process-local and does not persist data.
pub async fn create_storage() -> DashboardStorage {
    match std::env::var("DATABASE_URL") {
        Ok(database_url) => match PgPoolOptions::new().connect(&database_url).await {
            Ok(pool) => {
                println!("Dashboard storage: PostgreSQL");

                DashboardStorage::postgres(pool)
            }

            Err(error) => {
                eprintln!("PostgreSQL unavailable: {}", error);

                println!("Dashboard storage: in-memory");

                DashboardStorage::memory()
            }
        },

        Err(_) => {
            println!(
                "DATABASE_URL not set. \
                 Dashboard storage: in-memory"
            );

            DashboardStorage::memory()
        }
    }
}

/// Creates a PostgreSQL connection pool.
pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|error| sqlx::Error::Configuration(Box::new(error)))?;

    PgPoolOptions::new().connect(&database_url).await
}
