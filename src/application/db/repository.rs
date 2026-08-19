use crate::application::config::ApplicationConfig;
use sqlx::PgPool;
use uuid::Uuid;

pub struct ApplicationRepository {
    pool: PgPool,
}

impl ApplicationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn save(&self, config: &ApplicationConfig) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query(
            "INSERT INTO transfer (
                id,
                operation,
                file_size,
                file_size_unit,
                num_files
            )
            VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(&config.operation)
        .bind(config.file_size as i64)
        .bind(&config.file_size_unit)
        .bind(config.num_files as i32)
        .execute(&self.pool)
        .await?;

        Ok(id)
    }
}
