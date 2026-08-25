/**
 * Results page entry point.
 *
 * Coordinates result loading, rendering, user interactions,
 * and PDF report downloads.
 */

import {
    getToday,
    loadResults
} from "./api.js";

import {
    renderGroups,
    updateStatus
} from "./render.js";

const dateInput =
    document.getElementById(
        "results-date"
    );

const loadButton =
    document.getElementById(
        "load-results"
    );

const pdfButton =
    document.getElementById(
        "download-pdf"
    );

/**
 * Loads and displays results for the selected date.
 *
 * @param {string} date
 */
async function displayResults(date) {
    const data =
        await loadResults(date);

    renderGroups(
        data.groups
    );

    updateStatus(
        date
    );
}

/**
 * Displays an error in the results status area.
 *
 * @param {Error} error
 */
function handleLoadError(error) {
    console.error(error);

    document.getElementById(
        "status-message"
    ).textContent =
        "Error loading results.";
}

dateInput.value =
    getToday();

loadButton.addEventListener(
    "click",
    () => {
        const date =
            dateInput.value;

        if (!date) {
            return;
        }

        displayResults(
            date
        ).catch(
            handleLoadError
        );
    }
);

pdfButton.addEventListener(
    "click",
    () => {
        const date =
            dateInput.value;

        if (!date) {
            return;
        }

        window.location.href =
            `/results/pdf?date=${encodeURIComponent(
                date
            )}`;
    }
);

displayResults(
    dateInput.value
).catch(
    handleLoadError
);