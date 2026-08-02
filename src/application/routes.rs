use axum::{routing::get, Router};
use super::handlers;



pub fn create_router() -> Router {

    Router::new().route("/", get(handlers::home))

}