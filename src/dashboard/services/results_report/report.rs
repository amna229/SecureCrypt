//! Main evaluation report generation functions.
//!
//! This module provides the public functions used to generate
//! evaluation reports in HTML and PDF format.

use crate::dashboard::metrics::results_repository::ResultsData;

use headless_chrome::{Browser, LaunchOptions};

use std::{error::Error, fs};

use super::html::build_html_document;

/// Generates a PDF report containing the evaluation results.
///
/// The report is first generated as an HTML document and then
/// rendered by a headless Chromium browser. The resulting PDF
/// is stored in the project working directory as
/// `reporte_evaluacion.pdf`.
pub fn guardar_reporte_pdf(data: &ResultsData) -> Result<(), Box<dyn Error>> {
    let html_content = build_results_report_html(data, true);

    let browser = Browser::new(LaunchOptions::default())?;

    let tab = browser.new_tab()?;

    let data_url = format!(
        "data:text/html;charset=utf-8,{}",
        urlencoding::encode(&html_content)
    );

    tab.navigate_to(&data_url)?;
    tab.wait_until_navigated()?;

    let pdf_bytes = tab.print_to_pdf(None)?;

    fs::write("reporte_evaluacion.pdf", pdf_bytes)?;

    Ok(())
}

/// Builds the HTML representation of the evaluation results.
///
/// The generated report contains the experimental groups,
/// configurations, evaluation information, TLS handshake
/// metrics, transfer metrics, resource usage and derived
/// performance metrics.
pub fn build_results_report_html(data: &ResultsData, _for_pdf: bool) -> String {
    build_html_document(data)
}
