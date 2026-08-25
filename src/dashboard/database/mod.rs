use sqlx::{PgPool, postgres::PgPoolOptions};

/// Creates and initializes a PostgreSQL connection pool.
///
/// The database connection URL is obtained from the `DATABASE_URL`
/// environment variable. The pool is then used by the dashboard
/// to reuse database connections efficiently.
pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

    PgPoolOptions::new().connect(&database_url).await
}
