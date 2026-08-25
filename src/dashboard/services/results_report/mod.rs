//! Evaluation results reporting service.
//!
//! This module provides the functionality required to generate
//! human-readable reports from the results obtained during evaluations.
//!
//! The report generation process is divided into several modules:
//!
//! - `report` contains the main report generation functions.
//! - `html` contains the HTML document structure and styles.
//! - `sections` contains the different sections of the report.
//! - `formatting` contains formatting and HTML escaping utilities.

pub mod formatting;
pub mod html;
pub mod report;
pub mod sections;

pub use report::build_results_report_html;
