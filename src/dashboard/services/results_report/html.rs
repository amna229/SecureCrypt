//! HTML document generation.
//!
//! This module defines the structure and styles of the HTML
//! document used to display evaluation results.

use crate::dashboard::metrics::results_repository::ResultsData;

use super::sections::append_group;

/// Builds the complete HTML document.
///
/// The document contains the report title and all experimental
/// groups stored in the provided results data.
pub fn build_html_document(data: &ResultsData) -> String {
    let mut html = String::new();

    html.push_str(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<title>Evaluation Results</title>

<style>

body {
    font-family:
        Arial,
        Helvetica,
        sans-serif;
    font-size: 9pt;
    margin: 20px;
    padding: 0;
    color: #222;
    line-height: 1.2;
}

h1 {
    font-size: 18pt;
    margin-bottom: 12px;
}

h2 {
    font-size: 14pt;
    margin-top: 28px;
    margin-bottom: 8px;
    border-bottom:
        1px solid #ccc;
    padding-bottom: 4px;
}

h3 {
    font-size: 11pt;
    margin-top: 18px;
    margin-bottom: 6px;
}

.group {
    margin-bottom: 35px;
    page-break-inside: avoid;
}

.group-title {
    font-size: 13pt;
    font-weight: bold;
    margin-top: 24px;
    margin-bottom: 8px;
    padding: 8px 10px;
    background-color: #f2f2f2;
    border:
        1px solid #ccc;
    border-radius: 4px;
}

.configuration {
    margin-bottom: 14px;
    padding: 8px 10px;
    background-color: #fafafa;
    border:
        1px solid #ddd;
    font-size: 8.5pt;
}

.configuration span {
    font-weight: bold;
}

table {
    width: 100%;
    border-collapse:
        collapse;
    margin-top: 8px;
    margin-bottom: 14px;
}

th,
td {
    border:
        1px solid #ccc;
    padding:
        5px 4px;
    text-align:
        center;
    font-size: 8pt;
    white-space:
        nowrap;
}

th {
    background-color:
        #f2f2f2;
    font-weight:
        bold;
}

.metric-table {
    width: 100%;
    border-collapse:
        separate;
    border-spacing:
        8px;
    margin-top: 10px;
}

.metric-td {
    border:
        1px solid #ccc;
    padding: 8px;
    background-color:
        #fafafa;
    text-align: left;
    width: 50%;
}

.metric-title {
    font-size: 8pt;
    color: #666;
}

.metric-value {
    font-size: 12pt;
    font-weight: bold;
    margin-top: 3px;
}

.empty-message {
    color: #666;
    font-size: 8.5pt;
    margin:
        6px 0 12px 0;
}

@media print {

    body {
        margin: 0;
        padding: 0;
    }

    h2,
    h3 {
        page-break-after:
            avoid;
    }

    table {
        page-break-inside:
            avoid;
    }

    .group {
        page-break-inside:
            avoid;
    }

}

</style>

</head>

<body>
"#,
    );

    html.push_str("<h1>Evaluation Results</h1>");

    if data.groups.is_empty() {
        html.push_str(
            "<p class=\"empty-message\">\
             No recorded evaluations found.\
             </p>",
        );
    }

    for (index, group) in data.groups.iter().enumerate() {
        append_group(&mut html, index + 1, group);
    }

    html.push_str("</body></html>");

    html
}
