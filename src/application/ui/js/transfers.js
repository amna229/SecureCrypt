/**
 * Transfer management.
 *
 * This module builds transfer configurations, starts transfers,
 * and handles transfer-related Server-Sent Events.
 */

import { state } from "./state.js";

import {
    saveTransferConfiguration
} from "./api.js";

import {
    fileSize,
    fileSizeUnit,
    numFiles,
    statusText,
    showTransferStatus,
    updateTransferProgress,
    showTransferCompleted
} from "./ui.js";

/// Builds a transfer configuration from the current form values.
export function getConfiguration(
    evaluationId
) {
    return {
        evaluation_id:
            evaluationId,

        operation:
            document.querySelector(
                'input[name="operation"]:checked'
            ).value,

        file_size:
            Number(
                fileSize.value
            ),

        file_size_unit:
            fileSizeUnit.value,

        num_files:
            Number(
                numFiles.value
            )
    };
}

/// Starts a transfer using the provided evaluation identifier.
export async function startTransfer(
    evaluationId
) {
    const configuration =
        getConfiguration(
            evaluationId
        );

    state.currentOperation =
        configuration.operation;

    statusText.textContent =
        "Saving transfer...";

    const response =
        await saveTransferConfiguration(
            configuration
        );

    if (!response.ok) {
        statusText.textContent =
            "Error starting transfer";

        return false;
    }

    const result =
        await response.json();

    console.log(
        "Transfer ID:",
        result.transfer_id
    );

    showTransferStatus(
        configuration.operation
    );

    statusText.textContent =
        "Transfer started";

    return true;
}

/// Connects to the transfer SSE stream.
export function connectToTransferEvents() {
    if (state.transferEventSource) {
        state.transferEventSource.close();
    }

    state.transferEventSource =
        new EventSource(
            "http://127.0.0.1:3000/eval/transfer-events"
        );

    state.transferEventSource.addEventListener(
        "transfer-started",
        event => {
            try {
                const data =
                    JSON.parse(
                        event.data
                    );

                showTransferStatus(
                    data.operation
                );

                const transferMessage =
                    document.getElementById(
                        "transfer-message"
                    );

                const transferDetails =
                    document.getElementById(
                        "transfer-details"
                    );

                transferMessage.textContent =
                    data.operation === "upload"
                        ? "Uploading files..."
                        : "Downloading files...";

                transferDetails.textContent =
                    "Transfer started...";
            } catch (error) {
                console.error(
                    "Invalid transfer-started event:",
                    error
                );
            }
        }
    );

    state.transferEventSource.addEventListener(
        "transfer-completed",
        event => {
            try {
                const data =
                    JSON.parse(
                        event.data
                    );

                state.currentOperation =
                    data.operation ||
                    state.currentOperation;

                updateTransferProgress(
                    data.completed,
                    data.total
                );

                if (data.all_completed) {
                    showTransferCompleted(
                        data.completed,
                        data.total
                    );
                }
            } catch (error) {
                console.error(
                    "Invalid transfer-completed event:",
                    error
                );
            }
        }
    );

    state.transferEventSource.onerror =
        error => {
            console.error(
                "Transfer SSE connection error:",
                error
            );
        };
}