/**
 * Evaluation results rendering.
 *
 * This module transforms evaluation result data received
 * from the backend into HTML displayed by the results page.
 */

import {
    escapeHtml,
    formatMode,
    formatOperation,
    formatDate,
    formatBytes,
    formatDuration
} from "./formatting.js";

/**
 * Updates the status message shown above the results.
 *
 * @param {string} date
 */
export function updateStatus(date) {
    const status =
        document.getElementById(
            "status-message"
        );

    status.textContent =
        `Showing results for ${date}`;
}

/**
 * Renders all evaluation groups.
 *
 * @param {Array} groups
 */
export function renderGroups(groups) {
    const container =
        document.getElementById(
            "results-groups"
        );

    if (!groups || groups.length === 0) {
        container.innerHTML = `
            <div class="no-results">
                No recorded evaluations found for this date.
            </div>
        `;

        return;
    }

    container.innerHTML =
        groups
            .map(
                group =>
                    renderGroup(group)
            )
            .join("");
}

/**
 * Renders a single experimental group.
 *
 * @param {Object} group
 * @returns {string}
 */
function renderGroup(group) {
    const comparisonAvailable =
        hasBothModes(
            group.evaluations
        );

    return `
        <section class="group">
            <div class="group-header">
                <h2 class="group-title">
                    Evaluation Group
                </h2>

                <div class="configuration">
                    <div class="configuration-item">
                        <strong>Clients:</strong>
                        ${group.num_clients}
                    </div>

                    <div class="configuration-item">
                        <strong>Operation:</strong>
                        ${formatOperation(
                            group.operation
                        )}
                    </div>

                    <div class="configuration-item">
                        <strong>File size:</strong>
                        ${group.file_size}
                        ${escapeHtml(
                            group.file_size_unit
                        )}
                    </div>

                    <div class="configuration-item">
                        <strong>Files:</strong>
                        ${group.num_files}
                    </div>
                </div>
            </div>

            <div class="section">
                <h3>Evaluations</h3>
                ${renderEvaluations(
                    group.evaluations
                )}
            </div>

            <div class="section">
                <h3>TLS Handshake Summary</h3>
                ${renderHandshakes(
                    group.handshakes
                )}
            </div>

            <div class="section">
                <h3>Transfer Summary</h3>
                ${renderTransfers(
                    group.transfers
                )}
            </div>

            <div class="section">
                <h3>Resource Usage</h3>
                ${renderResources(
                    group.resources
                )}
            </div>

            <div class="section">
                <h3>Derived Metrics</h3>

                ${
                    comparisonAvailable
                        ? renderDerived(
                            group.derived
                        )
                        : `
                            <div class="comparison-not-available">
                                Derived metrics are not available for this group
                                because it does not contain both classical and
                                post-quantum evaluations.
                            </div>
                        `
                }
            </div>
        </section>
    `;
}

/**
 * Checks whether both classical and post-quantum
 * evaluations are available.
 *
 * @param {Array} evaluations
 * @returns {boolean}
 */
function hasBothModes(evaluations) {
    const hasClassical =
        evaluations.some(
            evaluation =>
                evaluation.crypto_mode ===
                "classical"
        );

    const hasPostQuantum =
        evaluations.some(
            evaluation =>
                evaluation.crypto_mode ===
                "post_quantum"
        );

    return (
        hasClassical &&
        hasPostQuantum
    );
}

/**
 * Renders evaluation summaries.
 *
 * @param {Array} evaluations
 * @returns {string}
 */
function renderEvaluations(
    evaluations
) {
    if (
        !evaluations ||
        evaluations.length === 0
    ) {
        return `
            <div class="empty-section">
                No evaluations recorded.
            </div>
        `;
    }

    return `
        <table>
            <thead>
                <tr>
                    <th>Mode</th>
                    <th>Clients</th>
                    <th>Started At</th>
                    <th>Finished At</th>
                    <th>Status</th>
                </tr>
            </thead>

            <tbody>
                ${evaluations
                    .map(
                        evaluation => `
                            <tr>
                                <td>
                                    ${formatMode(
                                        evaluation.crypto_mode
                                    )}
                                </td>

                                <td>
                                    ${evaluation.num_clients}
                                </td>

                                <td>
                                    ${formatDate(
                                        evaluation.started_at
                                    )}
                                </td>

                                <td>
                                    ${
                                        evaluation.finished_at
                                            ? formatDate(
                                                evaluation.finished_at
                                            )
                                            : "-"
                                    }
                                </td>

                                <td>
                                    ${escapeHtml(
                                        evaluation.status
                                    )}
                                </td>
                            </tr>
                        `
                    )
                    .join("")}
            </tbody>
        </table>
    `;
}

/**
 * Renders TLS handshake summaries.
 *
 * @param {Array} handshakes
 * @returns {string}
 */
function renderHandshakes(
    handshakes
) {
    if (
        !handshakes ||
        handshakes.length === 0
    ) {
        return `
            <div class="empty-section">
                No recorded handshakes found.
            </div>
        `;
    }

    return `
        <table>
            <thead>
                <tr>
                    <th>Mode</th>
                    <th>KX</th>
                    <th>Samples</th>
                    <th>Mean</th>
                    <th>Median</th>
                    <th>Std. Deviation</th>
                    <th>Min.</th>
                    <th>Max.</th>
                </tr>
            </thead>

            <tbody>
                ${handshakes
                    .map(
                        handshake => `
                            <tr>
                                <td>
                                    ${formatMode(
                                        handshake.crypto_mode
                                    )}
                                </td>

                                <td>
                                    ${escapeHtml(
                                        handshake.kx_group
                                    )}
                                </td>

                                <td>
                                    ${handshake.samples}
                                </td>

                                <td>
                                    ${handshake.mean_ms.toFixed(2)}
                                    ms
                                </td>

                                <td>
                                    ${handshake.median_ms.toFixed(2)}
                                    ms
                                </td>

                                <td>
                                    ${handshake.stddev_ms.toFixed(2)}
                                    ms
                                </td>

                                <td>
                                    ${handshake.min_ms}
                                    ms
                                </td>

                                <td>
                                    ${handshake.max_ms}
                                    ms
                                </td>
                            </tr>
                        `
                    )
                    .join("")}
            </tbody>
        </table>
    `;
}

/**
 * Renders transfer summaries.
 *
 * @param {Array} transfers
 * @returns {string}
 */
function renderTransfers(
    transfers
) {
    if (
        !transfers ||
        transfers.length === 0
    ) {
        return `
            <div class="empty-section">
                No recorded transfers found.
            </div>
        `;
    }

    return `
        <table>
            <thead>
                <tr>
                    <th>Mode</th>
                    <th>Samples</th>
                    <th>Mean Duration</th>
                    <th>Mean Throughput</th>
                    <th>Data Transferred</th>
                </tr>
            </thead>

            <tbody>
                ${transfers
                    .map(
                        transfer => `
                            <tr>
                                <td>
                                    ${formatMode(
                                        transfer.crypto_mode
                                    )}
                                </td>

                                <td>
                                    ${transfer.samples}
                                </td>

                                <td>
                                    ${formatDuration(
                                        transfer.mean_duration_ms
                                    )}
                                </td>

                                <td>
                                    ${transfer.mean_throughput_mbps.toFixed(2)}
                                    Mbps
                                </td>

                                <td>
                                    ${formatBytes(
                                        transfer.total_bytes
                                    )}
                                </td>
                            </tr>
                        `
                    )
                    .join("")}
            </tbody>
        </table>
    `;
}

/**
 * Renders container resource summaries.
 *
 * @param {Array} resources
 * @returns {string}
 */
function renderResources(
    resources
) {
    if (
        !resources ||
        resources.length === 0
    ) {
        return `
            <div class="empty-section">
                No recorded resource metrics found.
            </div>
        `;
    }

    return `
        <table>
            <thead>
                <tr>
                    <th>Mode</th>
                    <th>Role</th>
                    <th>Samples</th>
                    <th>Mean CPU</th>
                    <th>Peak CPU</th>
                    <th>Mean Peak RAM</th>
                    <th>Max RAM</th>
                </tr>
            </thead>

            <tbody>
                ${resources
                    .map(
                        resource => `
                            <tr>
                                <td>
                                    ${formatMode(
                                        resource.crypto_mode
                                    )}
                                </td>

                                <td>
                                    ${escapeHtml(
                                        resource.role
                                    )}
                                </td>

                                <td>
                                    ${resource.samples}
                                </td>

                                <td>
                                    ${resource.mean_cpu_avg_percent.toFixed(2)}
                                    %
                                </td>

                                <td>
                                    ${resource.max_cpu_peak_percent.toFixed(2)}
                                    %
                                </td>

                                <td>
                                    ${formatBytes(
                                        resource.mean_memory_peak_bytes
                                    )}
                                </td>

                                <td>
                                    ${formatBytes(
                                        resource.max_memory_peak_bytes
                                    )}
                                </td>
                            </tr>
                        `
                    )
                    .join("")}
            </tbody>
        </table>
    `;
}

/**
 * Renders derived comparison metrics.
 *
 * @param {Object} derived
 * @returns {string}
 */
function renderDerived(
    derived
) {
    const metrics = [
        [
            "Handshake overhead",
            derived.handshake_overhead_percent
        ],
        [
            "Transfer overhead",
            derived.transfer_overhead_percent
        ],
        [
            "CPU overhead",
            derived.cpu_overhead_percent
        ],
        [
            "Memory overhead",
            derived.memory_overhead_percent
        ],
        [
            "Throughput change",
            derived.throughput_change_percent
        ]
    ];

    return `
        <div class="metric-grid">
            ${metrics
                .map(
                    ([label, value]) => `
                        <div class="metric-card">
                            <div class="metric-label">
                                ${escapeHtml(label)}
                            </div>

                            <div class="metric-value">
                                ${
                                    value === null
                                        ? "-"
                                        : `${value.toFixed(2)} %`
                                }
                            </div>
                        </div>
                    `
                )
                .join("")}
        </div>
    `;
}