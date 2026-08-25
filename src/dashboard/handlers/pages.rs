//! Dashboard page handlers.
//!
//! This module contains the HTTP handlers responsible for serving
//! the dashboard user interface pages.

use axum::response::Html;

use crate::dashboard::ui::templates;

/// Serves the main dashboard page.
pub async fn dashboard() -> Html<&'static str> {
    Html(templates::DASHBOARD)
}

/// Serves the server configuration page.
pub async fn server() -> Html<&'static str> {
    Html(templates::SERVER)
}

/// Serves the client configuration page.
pub async fn client() -> Html<String> {
    let application_url =
        std::env::var("APPLICATION_URL").unwrap_or_else(|_| "https://127.0.0.1:8443".to_string());

    let html = templates::CLIENT.replace("{{APPLICATION_URL}}", &application_url);

    Html(html)
}

/// Serves the evaluation results page.
pub async fn results() -> Html<&'static str> {
    Html(templates::RESULTS)
}
