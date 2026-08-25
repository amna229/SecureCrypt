/**
 * Application API communication.
 *
 * This module contains HTTP requests made by the application
 * page to the dashboard and application backend.
 */

const DASHBOARD_URL =
    "http://127.0.0.1:3000";

/// Starts the evaluation environment.
export function startEvaluationRequest() {
    return fetch(
        `${DASHBOARD_URL}/eval/start`,
        {
            method: "POST"
        }
    );
}

/// Retrieves the currently active evaluation.
export async function getCurrentEvaluation() {
    const response =
        await fetch(
            `${DASHBOARD_URL}/eval/current`
        );

    if (!response.ok) {
        throw new Error(
            "Error obtaining evaluation ID"
        );
    }

    return response.json();
}

/// Creates a new transfer configuration.
export function saveTransferConfiguration(
    configuration
) {
    return fetch(
        "/application/start",
        {
            method: "POST",

            headers: {
                "Content-Type":
                    "application/json"
            },

            body:
                JSON.stringify(
                    configuration
                )
        }
    );
}

/// Stops the current evaluation environment.
export function stopEvaluationRequest() {
    return fetch(
        `${DASHBOARD_URL}/eval/stop`,
        {
            method: "POST"
        }
    );
}

/// Resets the current evaluation configuration.
export function resetEvaluationRequest() {
    return fetch(
        `${DASHBOARD_URL}/eval/reset`,
        {
            method: "POST"
        }
    );
}