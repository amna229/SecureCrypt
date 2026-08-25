/**
 * Application page UI management.
 *
 * This module contains the functions responsible for updating
 * the visual state of the application interface.
 */

import { state } from "./state.js";

export const startButton =
    document.getElementById(
        "start-transfer"
    );

export const statusText =
    document.getElementById(
        "status"
    );

export const fileSize =
    document.getElementById(
        "file-size"
    );

export const fileSizeUnit =
    document.getElementById(
        "file-size-unit"
    );

export const numFiles =
    document.getElementById(
        "num-files"
    );

export const resultsButton =
    document.getElementById(
        "results-button"
    );

export const stopEvaluationButton =
    document.getElementById(
        "stop-evaluation"
    );

export const reconfigureButton =
    document.getElementById(
        "reconfigure-button"
    );

const transferStatus =
    document.getElementById(
        "transfer-status"
    );

const transferMessage =
    document.getElementById(
        "transfer-message"
    );

const transferDetails =
    document.getElementById(
        "transfer-details"
    );

const progressFill =
    document.getElementById(
        "progress-fill"
    );

const evaluationStatus =
    document.getElementById(
        "evaluation-status"
    );

const evaluationMessage =
    document.getElementById(
        "evaluation-message"
    );

const evaluationDetails =
    document.getElementById(
        "evaluation-details"
    );

const evaluationProgressFill =
    document.getElementById(
        "evaluation-progress-fill"
    );

const evaluationProgress =
    document.getElementById(
        "evaluation-progress"
    );

/// Normalizes the numeric input values used by the application.
export function normalizeInputs() {
    if (fileSize.value < 1) {
        fileSize.value = 1;
    }

    if (numFiles.value < 1) {
        numFiles.value = 1;
    }
}

/// Displays the current transfer status.
export function showTransferStatus(
    operation
) {
    state.currentOperation =
        operation;

    transferStatus.classList.add(
        "visible"
    );

    progressFill.style.width =
        "0%";

    transferMessage.textContent =
        operation === "upload"
            ? "Uploading files..."
            : "Downloading files...";

    transferDetails.textContent =
        "Transfer starting...";
}

/// Hides the transfer status section.
export function hideTransferStatus() {
    transferStatus.classList.remove(
        "visible"
    );

    progressFill.style.width =
        "0%";

    state.currentOperation =
        null;
}

/// Displays the evaluation status section.
export function showEvaluationStatus() {
    evaluationStatus.classList.add(
        "visible"
    );

    evaluationMessage.textContent =
        "Starting evaluation environment...";

    evaluationDetails.textContent =
        "Preparing evaluation environment...";

    evaluationProgressFill.style.width =
        "0%";

    evaluationProgress.innerHTML =
        "";
}

/// Hides the evaluation status section.
export function hideEvaluationStatus() {
    evaluationStatus.classList.remove(
        "visible"
    );

    evaluationProgressFill.style.width =
        "0%";

    evaluationProgress.innerHTML =
        "";
}

/// Updates the evaluation status messages.
export function setEvaluationMessage(
    message,
    details
) {
    evaluationMessage.textContent =
        message;

    evaluationDetails.textContent =
        details;
}

/// Adds an item to the evaluation progress list.
export function addEvaluationItem(
    message
) {
    const item =
        document.createElement(
            "div"
        );

    item.className =
        "evaluation-progress-item";

    item.textContent =
        message;

    evaluationProgress.appendChild(
        item
    );

    evaluationProgress.scrollTop =
        evaluationProgress.scrollHeight;
}

/// Updates the evaluation progress bar.
export function updateEvaluationProgress(
    completed,
    total
) {
    state.evaluationTotalClients =
        total;

    const percentage =
        total > 0
            ? (
                completed /
                total
            ) * 100
            : 0;

    evaluationProgressFill.style.width =
        `${Math.min(
            100,
            percentage
        )}%`;
}

/// Resets the evaluation progress display.
export function clearEvaluationProgress() {
    evaluationProgress.innerHTML =
        "";

    evaluationProgressFill.style.width =
        "0%";
}

/// Updates the transfer progress information.
export function updateTransferProgress(
    completed,
    total
) {
    const percentage =
        total > 0
            ? (
                completed /
                total
            ) * 100
            : 0;

    progressFill.style.width =
        `${Math.min(
            100,
            percentage
        )}%`;

    transferDetails.textContent =
        `${completed} of ${total} client transfers completed`;
}

/// Marks the transfer as completed in the UI.
export function showTransferCompleted(
    completed,
    total
) {
    progressFill.style.width =
        "100%";

    transferMessage.textContent =
        "Transfer completed ✓";

    transferDetails.textContent =
        `${completed} of ${total} client transfers completed`;

    resultsButton.disabled =
        false;

    statusText.textContent =
        "Evaluation completed";

    setTimeout(
        hideTransferStatus,
        1200
    );
}