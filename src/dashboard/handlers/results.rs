use crate::dashboard::{
    database::storage::DashboardStorage,
    metrics::results_repository::{ResultsData, ResultsRepository},
    services::results_report::build_results_report_html,
    state::DashboardState,
};

use axum::{
    body::Body,
    extract::{Query, State},
    http::{StatusCode, header},
    response::Response,
};

use chrono::{Local, NaiveDate};

use headless_chrome::{Browser, LaunchOptions};

use std::sync::Arc;

#[derive(Debug, serde::Deserialize)]
pub struct ResultsQuery {
    pub date: Option<String>,
}

/// Returns the current date used by the results endpoint.
fn get_current_date(state: &DashboardState) -> NaiveDate {
    match &state.db {
        DashboardStorage::Postgres(_) | DashboardStorage::Memory(_) => Local::now().date_naive(),
    }
}

/// Returns the evaluation results for the requested date.
pub async fn results_data(
    State(state): State<Arc<DashboardState>>,
    Query(query): Query<ResultsQuery>,
) -> Result<axum::Json<ResultsData>, StatusCode> {
    let date = match query.date {
        Some(value) => {
            NaiveDate::parse_from_str(&value, "%Y-%m-%d").map_err(|_| StatusCode::BAD_REQUEST)?
        }

        None => get_current_date(&state),
    };

    let repository = ResultsRepository::new(state.db.clone());

    repository
        .load(date)
        .await
        .map(axum::Json)
        .map_err(|error| {
            eprintln!("Error loading results: {}", error);

            StatusCode::INTERNAL_SERVER_ERROR
        })
}

/// Generates a PDF report containing the evaluation results.
pub async fn results_pdf(
    State(state): State<Arc<DashboardState>>,
    Query(query): Query<ResultsQuery>,
) -> Result<Response, StatusCode> {
    let date = match query.date {
        Some(value) => {
            NaiveDate::parse_from_str(&value, "%Y-%m-%d").map_err(|_| StatusCode::BAD_REQUEST)?
        }

        None => get_current_date(&state),
    };

    let repository = ResultsRepository::new(state.db.clone());

    let data = repository.load(date).await.map_err(|error| {
        eprintln!("Error loading results for PDF: {}", error);

        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let html = build_results_report_html(&data, true);

    let pdf_bytes = tokio::task::spawn_blocking(move || {
        let args = vec![
            std::ffi::OsStr::new("--no-sandbox"),
            std::ffi::OsStr::new("--disable-setuid-sandbox"),
            std::ffi::OsStr::new("--disable-gpu"),
        ];

        let options = LaunchOptions::default_builder()
            .args(args)
            .build()
            .map_err(|error| error.to_string())?;

        let browser = Browser::new(options)?;

        let tab = browser.new_tab()?;

        let data_url = format!(
            "data:text/html;charset=utf-8,{}",
            urlencoding::encode(&html)
        );

        tab.navigate_to(&data_url)?;
        tab.wait_until_navigated()?;

        let bytes = tab.print_to_pdf(None)?;

        Ok::<Vec<u8>, Box<dyn std::error::Error + Send + Sync>>(bytes)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .map_err(|error| {
        eprintln!(
            "Error generating PDF with \
                 headless_chrome: {}",
            error
        );

        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; \
             filename=\"securecrypt_results.pdf\"",
        )
        .body(Body::from(pdf_bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}
