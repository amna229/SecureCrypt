/**
 * Evaluation lifecycle management.
 *
 * This module controls starting, stopping and resetting the
 * evaluation environment and processes evaluation lifecycle events.
 */

import { state } from "./state.js";

import {
    startEvaluationRequest,
    getCurrentEvaluation,
    stopEvaluationRequest,
    resetEvaluationRequest
} from "./api.js";

import {
    startButton,
    resultsButton,
    stopEvaluationButton,
    reconfigureButton,
    statusText,
    fileSize,
    fileSizeUnit,
    numFiles,
    showEvaluationStatus,
    hideEvaluationStatus,
    setEvaluationMessage,
    addEvaluationItem,
    updateEvaluationProgress,
    clearEvaluationProgress
} from "./ui.js";

/// Starts the evaluation environment and obtains its identifier.
export async function startEvaluation() {

    if (!state.evaluationStarted) {

        state.redirectToResultsAfterStop = false;

        showEvaluationStatus();

        statusText.textContent = "Starting evaluation environment...";

        startButton.disabled = true;

        resultsButton.disabled = true;

        const response = await startEvaluationRequest();

        if (!response.ok) {

            startButton.disabled = false;

            hideEvaluationStatus();

            statusText.textContent = "Error starting evaluation";

            return null;
        }

        state.evaluationStarted = true;

        stopEvaluationButton.disabled = false;

        reconfigureButton.disabled = true;

        resultsButton.disabled = true;
    }

    const evaluationData = await getCurrentEvaluation();

    const evaluationId = evaluationData.evaluation_id;

    state.currentEvaluationId = evaluationId;

    if (!evaluationId) {

        statusText.textContent = "No active evaluation found";

        return null;
    }

    return evaluationId;
}

/// Stops the current evaluation environment.
export async function stopEvaluation(redirectAfterStop = false) {

    state.redirectToResultsAfterStop = redirectAfterStop;

    statusText.textContent = "Stopping evaluation...";

    stopEvaluationButton.disabled = true;

    startButton.disabled = true;

    showEvaluationStatus();

    setEvaluationMessage(
        "Stopping evaluation...",
        "Stopping evaluation environment..."
    );

    clearEvaluationProgress();

    const response = await stopEvaluationRequest();

    if (!response.ok) {

        stopEvaluationButton.disabled = false;

        startButton.disabled = false;

        statusText.textContent = "Error stopping evaluation";

        state.redirectToResultsAfterStop = false;

        return;
    }

    statusText.textContent = "Stopping evaluation...";
}

/// Resets the current evaluation configuration.
export async function reconfigure() {

    const response = await resetEvaluationRequest();

    if (!response.ok) {
        statusText.textContent = "Error resetting configuration";

        return;
    }

    state.evaluationStarted = false;

    state.currentEvaluationId = null;

    state.redirectToResultsAfterStop = false;

    stopEvaluationButton.disabled = true;

    reconfigureButton.disabled = true;

    resultsButton.disabled = true;

    startButton.disabled = false;

    hideEvaluationStatus();

    document
        .querySelector(
            'input[name="operation"][value="upload"]'
        )
        .checked = true;

    fileSize.value = 1;

    fileSizeUnit.value = "KB";

    numFiles.value = 1;

    statusText.textContent = "Ready to configure a new evaluation";

    window.location.href = "http://127.0.0.1:3000/server";
}

/// Connects to the evaluation lifecycle SSE stream.
export function connectToEvaluationEvents() {
    if (state.evaluationEventSource) {

        state.evaluationEventSource.close();
    }

    state.evaluationEventSource =
        new EventSource(
            "http://127.0.0.1:3000/eval/evaluation-events"
        );

    const eventTypes = [
        "environment-starting",
        "server-started",
        "client-started",
        "environment-ready",
        "environment-stopping",
        "client-stopped",
        "server-stopped",
        "environment-stopped"
    ];

    eventTypes.forEach(
        eventType => {
            state.evaluationEventSource.addEventListener(
                eventType,
                event => {
                    try {
                        const data =
                            JSON.parse(
                                event.data
                            );

                        handleEvaluationEvent(
                            data
                        );
                    } catch (error) {
                        console.error(
                            `Invalid ${eventType} event:`,
                            error
                        );
                    }
                }
            );
        }
    );

    state.evaluationEventSource.onerror =
        error => {
            console.error(
                "Evaluation SSE connection error:",
                error
            );
        };
}

/// Processes an evaluation lifecycle event.
function handleEvaluationEvent(data) {
    const {
        event_type,
        evaluation_id,
        client_id,
        completed,
        total
    } = data;

    if (
        state.currentEvaluationId &&
        evaluation_id !==
        state.currentEvaluationId
    ) {
        return;
    }

    if (
        !state.currentEvaluationId &&
        evaluation_id
    ) {
        state.currentEvaluationId =
            evaluation_id;
    }

    switch (event_type) {
        case "environment-starting":
            showEvaluationStatus();

            setEvaluationMessage(
                "Starting evaluation environment...",
                "Preparing evaluation environment..."
            );

            updateEvaluationProgress(
                completed,
                total
            );

            break;

        case "server-started":
            showEvaluationStatus();

            setEvaluationMessage(
                "Starting evaluation environment...",
                "Server started"
            );

            addEvaluationItem(
                "✓ Server started"
            );

            break;

        case "client-started":
            showEvaluationStatus();

            setEvaluationMessage(
                "Starting evaluation environment...",
                `${completed} of ${total} clients started`
            );

            updateEvaluationProgress(
                completed,
                total
            );

            if (client_id) {
                addEvaluationItem(
                    `✓ Client ${client_id} started`
                );
            }

            break;

        case "environment-ready":
            showEvaluationStatus();

            setEvaluationMessage(
                "Environment ready ✓",
                `${total} of ${total} clients started`
            );

            updateEvaluationProgress(
                total,
                total
            );

            addEvaluationItem(
                "✓ Evaluation environment ready"
            );

            state.evaluationStarted =
                true;

            startButton.disabled =
                false;

            stopEvaluationButton.disabled =
                false;

            reconfigureButton.disabled =
                true;

            statusText.textContent =
                "Evaluation running";

            break;

        case "environment-stopping":
            showEvaluationStatus();

            setEvaluationMessage(
                "Stopping evaluation...",
                "Stopping evaluation environment..."
            );

            clearEvaluationProgress();

            break;

        case "client-stopped":
            showEvaluationStatus();

            setEvaluationMessage(
                "Stopping evaluation...",
                `${completed} of ${total} clients stopped`
            );

            updateEvaluationProgress(
                completed,
                total
            );

            if (client_id) {
                addEvaluationItem(
                    `✓ Client ${client_id} stopped`
                );
            }

            break;

        case "server-stopped":
            showEvaluationStatus();

            setEvaluationMessage(
                "Stopping evaluation...",
                "Server stopped"
            );

            addEvaluationItem(
                "✓ Server stopped"
            );

            break;

        case "environment-stopped":
            showEvaluationStatus();

            setEvaluationMessage(
                "Evaluation stopped ✓",
                "Evaluation environment cleaned up"
            );

            updateEvaluationProgress(
                total,
                total
            );

            addEvaluationItem(
                "✓ Evaluation environment stopped"
            );

            state.evaluationStarted = false;

            state.currentEvaluationId = null;

            stopEvaluationButton.disabled = true;

            reconfigureButton.disabled = false;

            startButton.disabled = false;

            resultsButton.disabled = false;

            statusText.textContent = "Evaluation stopped";

            if (state.redirectToResultsAfterStop) {
                state.redirectToResultsAfterStop = false;

                window.location.href = "http://127.0.0.1:3000/results";
            }

            break;

        default: break;
    }
}