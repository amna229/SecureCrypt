/**
 * Results API functions.
 *
 * This module handles communication with the dashboard
 * backend for retrieving evaluation results.
 */

/**
 * Returns today's date in YYYY-MM-DD format.
 *
 * @returns {string}
 */
export function getToday() {
    const now = new Date();

    const year =
        now.getFullYear();

    const month =
        String(
            now.getMonth() + 1
        ).padStart(2, "0");

    const day =
        String(
            now.getDate()
        ).padStart(2, "0");

    return `${year}-${month}-${day}`;
}

/**
 * Loads evaluation results for the specified date.
 *
 * @param {string} date
 * @returns {Promise<Object>}
 */
export async function loadResults(date) {
    const response =
        await fetch(
            `/results/data?date=${encodeURIComponent(date)}`
        );

    if (!response.ok) {
        throw new Error(
            "Could not load results"
        );
    }

    return response.json();
}