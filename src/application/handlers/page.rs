//! Application page handler.
//!
//! This module serves the main application interface.

use axum::response::Html;

/// Serves the main application page.
pub async fn my_app() -> Html<&'static str> {
    Html(include_str!("../ui/templates/myApp.html"))
}
