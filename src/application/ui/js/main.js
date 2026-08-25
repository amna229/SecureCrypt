/**
 * Application page entry point.
 *
 * Initializes the application interface, connects event streams,
 * and registers the page event handlers.
 */

import { state } from "./state.js";

import {
    fileSize,
    numFiles,
    resultsButton,
    startButton,
    stopEvaluationButton,
    reconfigureButton,
    normalizeInputs,
    hideTransferStatus
} from "./ui.js";

import {
    startEvaluation,
    stopEvaluation,
    reconfigure,
    connectToEvaluationEvents
} from "./evaluation.js";

import {
    startTransfer,
    connectToTransferEvents
} from "./transfers.js";

/// Starts the evaluation when necessary and then starts the transfer.
async function handleStart() {
    normalizeInputs();

    const evaluationId =
        await startEvaluation();

    if (!evaluationId) {
        return;
    }

    const success =
        await startTransfer(
            evaluationId
        );

    if (!success) {
        return;
    }
}

/// Initializes the application page.
function initialize() {
    resultsButton.disabled =
        true;

    fileSize.addEventListener(
        "input",
        normalizeInputs
    );

    numFiles.addEventListener(
        "input",
        normalizeInputs
    );

    startButton.addEventListener(
        "click",
        handleStart
    );

    resultsButton.addEventListener(
        "click",
        async () => {
            if (!state.evaluationStarted) {
                window.location.href =
                    "http://127.0.0.1:3000/results";

                return;
            }

            await stopEvaluation(true);
        }
    );

    stopEvaluationButton.addEventListener(
        "click",
        () => {
            stopEvaluation(false);
        }
    );

    reconfigureButton.addEventListener(
        "click",
        reconfigure
    );

    connectToTransferEvents();

    connectToEvaluationEvents();
}

initialize();