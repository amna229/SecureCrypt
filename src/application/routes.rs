use crate::application::handlers::{get_latest_transfer, get_transfer, my_app, start_app};
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::services::ServeDir;

pub fn create_router(pool: sqlx::PgPool) -> Router {
    Router::new()
        .route("/", get(my_app))
        .route("/application/start", post(start_app))
        .route("/transfer/{id}", get(get_transfer))
        .route("/transfer/latest", get(get_latest_transfer))
        .nest_service("/application-static", ServeDir::new("src/application/ui"))
        .with_state(pool)
}
