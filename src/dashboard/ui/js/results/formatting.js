/**
 * Formatting utilities for evaluation results.
 *
 * This module provides functions used to format result values
 * and safely insert dynamic data into generated HTML.
 */

/**
 * Escapes HTML-sensitive characters.
 *
 * This prevents values received from the backend from being
 * interpreted as HTML markup when inserted into the page.
 *
 * @param {*} value
 * @returns {string}
 */
export function escapeHtml(value) {
    return String(value)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;")
        .replace(/'/g, "&#39;");
}

/**
 * Formats a cryptographic mode for display.
 *
 * @param {string} mode
 * @returns {string}
 */
export function formatMode(mode) {
    if (mode === "post_quantum") {
        return "Post-Quantum";
    }

    if (mode === "classical") {
        return "Classical";
    }

    return escapeHtml(mode);
}

/**
 * Formats an evaluation operation for display.
 *
 * @param {string} operation
 * @returns {string}
 */
export function formatOperation(operation) {
    if (!operation) {
        return "-";
    }

    const formatted =
        operation.charAt(0).toUpperCase()
        + operation.slice(1);

    return escapeHtml(formatted);
}

/**
 * Formats a date value using the English locale.
 *
 * @param {string} value
 * @returns {string}
 */
export function formatDate(value) {
    return new Date(value).toLocaleString(
        "en-GB"
    );
}

/**
 * Converts a byte value into a human-readable representation.
 *
 * @param {number} bytes
 * @returns {string}
 */
export function formatBytes(bytes) {
    if (bytes < 1024) {
        return `${Math.round(bytes)} B`;
    }

    if (bytes < 1024 ** 2) {
        return `${(
            bytes / 1024
        ).toFixed(2)} KB`;
    }

    if (bytes < 1024 ** 3) {
        return `${(
            bytes / 1024 ** 2
        ).toFixed(2)} MB`;
    }

    return `${(
        bytes / 1024 ** 3
    ).toFixed(2)} GB`;
}

/**
 * Formats a duration in milliseconds.
 *
 * @param {number} milliseconds
 * @returns {string}
 */
export function formatDuration(milliseconds) {
    if (milliseconds < 1000) {
        return `${milliseconds.toFixed(2)} ms`;
    }

    return `${(
        milliseconds / 1000
    ).toFixed(2)} s`;
}