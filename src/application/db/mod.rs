//! Application database access.
//!
//! This module initializes the PostgreSQL connection pool and
//! exposes the application database repositories.

pub mod repository;

use sqlx::{PgPool, postgres::PgPoolOptions};

/// Creates the PostgreSQL connection pool and applies pending migrations.
///
/// The connection string is read from the `DATABASE_URL` environment variable.
pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

    let pool = PgPoolOptions::new().connect(&database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
