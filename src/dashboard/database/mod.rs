use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

    PgPoolOptions::new().connect(&database_url).await
}
