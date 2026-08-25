//! Formatting utilities for evaluation reports.
//!
//! This module contains helper functions used to format values
//! and safely insert evaluation data into the generated HTML.

/// Adds a single derived metric to the HTML report.
///
/// If the metric is not available, a dash is displayed instead.
pub fn add_metric_cell(html: &mut String, title: &str, value: Option<f64>) {
    let value_text = match value {
        Some(value) => {
            format!("{value:.2} %")
        }
        None => "-".to_string(),
    };

    html.push_str(&format!(
        r#"<td class="metric-td">
<div class="metric-title">{}</div>
<div class="metric-value">{}</div>
</td>"#,
        escape_html(title),
        value_text,
    ));
}

/// Converts a byte value into a human-readable representation.
///
/// Values are displayed using bytes, kilobytes, megabytes
/// or gigabytes depending on their magnitude.
pub fn format_bytes(bytes: f64) -> String {
    if bytes < 1024.0 {
        return format!("{bytes:.0} B");
    }

    if bytes < 1024.0 * 1024.0 {
        return format!("{:.2} KB", bytes / 1024.0);
    }

    if bytes < 1024.0 * 1024.0 * 1024.0 {
        return format!("{:.2} MB", bytes / (1024.0 * 1024.0));
    }

    format!("{:.2} GB", bytes / (1024.0 * 1024.0 * 1024.0))
}

/// Escapes HTML-sensitive characters.
///
/// This prevents values originating from evaluation data
/// from being interpreted as HTML markup when inserted
/// into the generated report.
pub fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
