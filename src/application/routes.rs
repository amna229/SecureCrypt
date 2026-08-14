use axum::{
    routing::{get, post},
    Router
};
use crate::application::{
    handlers::{
        my_app,
        start_app
    }
};
use tower_http::services::ServeDir;



pub fn create_router(pool: sqlx::PgPool) -> Router {

    Router::new()
    .route("/", get(my_app))
    .route("/application/start", post(start_app))
    .nest_service("/application-static", ServeDir::new("src/application/ui"))
    .with_state(pool)

}