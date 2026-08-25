//! Report section generation.
//!
//! This module contains the functions responsible for generating
//! the individual sections of the evaluation report.

use crate::dashboard::metrics::results_repository::{DerivedMetrics, EvaluationGroup};

use super::formatting::{add_metric_cell, escape_html, format_bytes};

/// Appends one experimental group and all its associated
/// measurements to the generated HTML document.
pub fn append_group(html: &mut String, group_number: usize, group: &EvaluationGroup) {
    html.push_str("<div class=\"group\">");

    html.push_str(&format!(
        "<div class=\"group-title\">\
             Experimental Group {}\
             </div>",
        group_number
    ));

    html.push_str(r#"<div class="configuration">"#);

    html.push_str(&format!(
        "<span>Clients:</span> {} &nbsp;&nbsp;\
         <span>Operation:</span> {} &nbsp;&nbsp;\
         <span>File size:</span> {} {} &nbsp;&nbsp;\
         <span>Files:</span> {}",
        group.num_clients,
        escape_html(&group.operation),
        group.file_size,
        escape_html(&group.file_size_unit),
        group.num_files,
    ));

    html.push_str("</div>");

    html.push_str("<h2>Evaluations</h2>");

    if group.evaluations.is_empty() {
        html.push_str(
            "<p class=\"empty-message\">\
             No evaluations recorded.\
             </p>",
        );
    } else {
        html.push_str(
            r#"<table>
<thead>
<tr>
<th>Crypto mode</th>
<th>Clients</th>
<th>Started at</th>
<th>Finished at</th>
<th>Status</th>
</tr>
</thead>
<tbody>
"#,
        );

        for evaluation in &group.evaluations {
            html.push_str(&format!(
                r#"<tr>
<td>{}</td>
<td>{}</td>
<td>{}</td>
<td>{}</td>
<td>{}</td>
</tr>
"#,
                escape_html(&evaluation.crypto_mode),
                evaluation.num_clients,
                evaluation.started_at,
                evaluation
                    .finished_at
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                escape_html(&evaluation.status),
            ));
        }

        html.push_str("</tbody></table>");
    }

    html.push_str("<h2>TLS Handshake</h2>");

    if group.handshakes.is_empty() {
        html.push_str(
            "<p class=\"empty-message\">\
             No recorded handshakes.\
             </p>",
        );
    } else {
        html.push_str(
            r#"<table>
<thead>
<tr>
<th>Crypto mode</th>
<th>KX group</th>
<th>Samples</th>
<th>Mean</th>
<th>Median</th>
<th>Std. dev</th>
<th>Min</th>
<th>Max</th>
</tr>
</thead>
<tbody>
"#,
        );

        for handshake in &group.handshakes {
            html.push_str(&format!(
                r#"<tr>
<td>{}</td>
<td>{}</td>
<td>{}</td>
<td>{:.2} ms</td>
<td>{:.2} ms</td>
<td>{:.2} ms</td>
<td>{} ms</td>
<td>{} ms</td>
</tr>
"#,
                escape_html(&handshake.crypto_mode),
                escape_html(&handshake.kx_group),
                handshake.samples,
                handshake.mean_ms,
                handshake.median_ms,
                handshake.stddev_ms,
                handshake.min_ms,
                handshake.max_ms,
            ));
        }

        html.push_str("</tbody></table>");
    }

    html.push_str("<h2>Transfers</h2>");

    if group.transfers.is_empty() {
        html.push_str(
            "<p class=\"empty-message\">\
             No recorded transfers.\
             </p>",
        );
    } else {
        html.push_str(
            r#"<table>
<thead>
<tr>
<th>Crypto mode</th>
<th>KX group</th>
<th>Samples</th>
<th>Mean duration</th>
<th>Mean throughput</th>
<th>Total data</th>
</tr>
</thead>
<tbody>
"#,
        );

        for transfer in &group.transfers {
            html.push_str(&format!(
                r#"<tr>
<td>{}</td>
<td>{}</td>
<td>{}</td>
<td>{:.2} s</td>
<td>{:.2} Mbps</td>
<td>{}</td>
</tr>
"#,
                escape_html(&transfer.crypto_mode),
                escape_html(&transfer.kx_group),
                transfer.samples,
                transfer.mean_duration_ms / 1000.0,
                transfer.mean_throughput_mbps,
                format_bytes(transfer.total_bytes as f64),
            ));
        }

        html.push_str("</tbody></table>");
    }

    html.push_str("<h2>Resource Usage</h2>");

    if group.resources.is_empty() {
        html.push_str(
            "<p class=\"empty-message\">\
             No recorded resource metrics.\
             </p>",
        );
    } else {
        html.push_str(
            r#"<table>
<thead>
<tr>
<th>Crypto mode</th>
<th>Role</th>
<th>Samples</th>
<th>Avg CPU</th>
<th>Peak CPU</th>
<th>Avg peak memory</th>
<th>Max memory</th>
</tr>
</thead>
<tbody>
"#,
        );

        for resource in &group.resources {
            html.push_str(&format!(
                r#"<tr>
<td>{}</td>
<td>{}</td>
<td>{}</td>
<td>{:.2} %</td>
<td>{:.2} %</td>
<td>{}</td>
<td>{}</td>
</tr>
"#,
                escape_html(&resource.crypto_mode),
                escape_html(&resource.role),
                resource.samples,
                resource.mean_cpu_avg_percent,
                resource.max_cpu_peak_percent,
                format_bytes(resource.mean_memory_peak_bytes),
                format_bytes(resource.max_memory_peak_bytes as f64,),
            ));
        }

        html.push_str("</tbody></table>");
    }

    html.push_str("<h2>Derived Metrics</h2>");

    append_derived_metrics(html, &group.derived);

    html.push_str("</div>");
}

/// Appends the derived performance metrics to the report.
///
/// These metrics summarize the relative overhead or change
/// introduced by the evaluated cryptographic configuration.
fn append_derived_metrics(html: &mut String, derived: &DerivedMetrics) {
    html.push_str(r#"<table class="metric-table">"#);

    html.push_str("<tr>");

    add_metric_cell(
        html,
        "Handshake overhead",
        derived.handshake_overhead_percent,
    );

    add_metric_cell(html, "Transfer overhead", derived.transfer_overhead_percent);

    html.push_str("</tr>");

    html.push_str("<tr>");

    add_metric_cell(html, "CPU overhead", derived.cpu_overhead_percent);

    add_metric_cell(html, "Memory overhead", derived.memory_overhead_percent);

    html.push_str("</tr>");

    html.push_str("<tr>");

    add_metric_cell(html, "Throughput change", derived.throughput_change_percent);

    html.push_str("<td style='border:none;background:none;'></td>");

    html.push_str("</tr></table>");
}
