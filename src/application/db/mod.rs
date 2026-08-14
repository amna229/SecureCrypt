pub mod repository;

use sqlx::{postgres::PgPoolOptions, PgPool};




pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");

    let pool = PgPoolOptions::new().connect(&database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}