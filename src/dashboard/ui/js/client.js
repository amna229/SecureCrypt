/**
 * Client configuration interface.
 *
 * Handles cryptographic configuration, key exchange selection,
 * connection validation, configuration persistence, evaluation
 * control and access to evaluation results.
 */

const classicalRadio =
    document.querySelector(
        'input[value="classical"]'
    );

const postQuantumRadio =
    document.querySelector(
        'input[value="post_quantum"]'
    );

const classicalKx =
    document.getElementById(
        "classical-kx"
    );

const postQuantumKx =
    document.getElementById(
        "post-quantum-kx"
    );

const cipherCheckboxes =
    document.querySelectorAll(
        'input[name="cipher_suites"]'
    );

const serverAddr =
    document.getElementById(
        "server-addr"
    );

const serverName =
    document.getElementById(
        "server-name"
    );

const numConnections =
    document.getElementById(
        "num-connections"
    );

const saveButton =
    document.getElementById(
        "save-button"
    );

const applicationButton =
    document.getElementById(
        "application-button"
    );

const resultsButton =
    document.getElementById(
        "results-button"
    );

const cipherError =
    document.getElementById(
        "cipher-error"
    );

const kxError =
    document.getElementById(
        "kx-error"
    );

let evaluationRunning =
    false;

let evaluationFinished =
    false;

/**
 * Returns the key exchange container corresponding
 * to the currently selected cryptographic mode.
 */
function getActiveKxContainer() {
    return classicalRadio.checked
        ? classicalKx
        : postQuantumKx;
}

/**
 * Returns the selected values from a group of checkboxes.
 *
 * @param {Element} container
 * @param {string} selector
 * @returns {string[]}
 */
function getSelectedValues(
    container,
    selector
) {
    return Array.from(
        container.querySelectorAll(
            `${selector}:checked`
        )
    ).map(
        checkbox => checkbox.value
    );
}

/**
 * Updates the visible key exchange options according
 * to the selected cryptographic mode.
 */
function updateKxGroups() {
    const isClassical =
        classicalRadio.checked;

    classicalKx.style.display =
        isClassical
            ? "block"
            : "none";

    postQuantumKx.style.display =
        isClassical
            ? "none"
            : "block";

    updateValidation();
}

/**
 * Validates the current client configuration.
 */
function updateValidation() {
    const selectedCiphers =
        getSelectedValues(
            document,
            'input[name="cipher_suites"]'
        );

    const activeKxContainer =
        getActiveKxContainer();

    const selectedKx =
        getSelectedValues(
            activeKxContainer,
            'input[name="kx_groups"]'
        );

    const noCiphersSelected =
        selectedCiphers.length === 0;

    const noKxSelected =
        selectedKx.length === 0;

    const invalidNumConnections =
        numConnections.value < 1 ||
        numConnections.value > 1000;

    cipherError.classList.toggle(
        "visible",
        noCiphersSelected
    );

    kxError.classList.toggle(
        "visible",
        noKxSelected
    );

    saveButton.disabled =
        noCiphersSelected ||
        noKxSelected ||
        invalidNumConnections ||
        evaluationRunning;
}

/**
 * Keeps the number of client connections within
 * the supported range.
 */
function normalizeConnections() {
    if (
        numConnections.value > 1000
    ) {
        numConnections.value = 1000;
    }

    if (
        numConnections.value < 1
    ) {
        numConnections.value = 1;
    }
}

/**
 * Builds the client configuration object.
 *
 * @returns {Object}
 */
function getConfiguration() {
    return {
        key_exchange:
            document.querySelector(
                'input[name="key_exchange"]:checked'
            ).value,

        cipher_suites:
            getSelectedValues(
                document,
                'input[name="cipher_suites"]'
            ),

        kx_groups:
            getSelectedValues(
                getActiveKxContainer(),
                'input[name="kx_groups"]'
            ),

        server_addr:
            serverAddr.value.trim(),

        server_name:
            serverName.value.trim(),

        num_connections:
            Number(
                numConnections.value
            )
    };
}

/**
 * Sends the current client configuration
 * to the dashboard backend.
 */
async function saveConfiguration() {
    const config =
        getConfiguration();

    const response =
        await fetch(
            "/info/client",
            {
                method: "POST",

                headers: {
                    "Content-Type":
                        "application/json"
                },

                body:
                    JSON.stringify(config)
            }
        );

    console.log(
        "Server response:",
        response.status
    );

    await checkClientConfiguration();
}

/**
 * Checks whether a client configuration has been stored.
 */
async function checkClientConfiguration() {
    const response =
        await fetch(
            "/info/client/status"
        );

    const configured =
        await response.json();

    if (!configured) {
        applicationButton.disabled =
            true;

        return;
    }

    if (!evaluationRunning) {
        applicationButton.disabled =
            false;
    }
}

/**
 * Starts the evaluation.
 *
 * The browser does not navigate to port 8443.
 */
async function startEvaluation() {
    applicationButton.disabled =
        true;

    resultsButton.disabled =
        true;

    evaluationRunning =
        true;

    evaluationFinished =
        false;

    saveButton.disabled =
        true;

    try {
        const response =
            await fetch(
                "/eval/start",
                {
                    method: "POST"
                }
            );

        if (!response.ok) {
            const errorText =
                await response.text();

            throw new Error(
                errorText ||
                "Failed to start evaluation"
            );
        }

        console.log(
            "Evaluation started"
        );

    } catch (error) {
        console.error(
            "Error starting evaluation:",
            error
        );

        evaluationRunning =
            false;

        applicationButton.disabled =
            false;

        updateValidation();

        alert(
            "The evaluation could not be started."
        );
    }
}

/**
 * Monitors evaluation lifecycle events sent by
 * the dashboard through Server-Sent Events.
 */
function listenForEvaluationEvents() {
    const eventSource =
        new EventSource(
            "/eval/evaluation-events"
        );

    eventSource.addEventListener(
        "environment-stopped",
        () => {
            evaluationRunning =
                false;

            evaluationFinished =
                true;

            applicationButton.disabled =
                true;

            resultsButton.disabled =
                false;

            updateValidation();

            alert(
                "The evaluation has finished."
            );
        }
    );

    eventSource.addEventListener(
        "environment-ready",
        () => {
            evaluationRunning =
                true;
        }
    );

    eventSource.onerror =
        error => {
            console.error(
                "Evaluation event connection error:",
                error
            );
        };
}

/**
 * Opens the results page once the evaluation
 * has completed.
 */
function openResults() {
    if (!evaluationFinished) {
        return;
    }

    window.location.href =
        "/results";
}

numConnections.addEventListener(
    "input",
    () => {
        normalizeConnections();
        updateValidation();
    }
);

classicalRadio.addEventListener(
    "change",
    updateKxGroups
);

postQuantumRadio.addEventListener(
    "change",
    updateKxGroups
);

cipherCheckboxes.forEach(
    checkbox => {
        checkbox.addEventListener(
            "change",
            updateValidation
        );
    }
);

document
    .querySelectorAll(
        'input[name="kx_groups"]'
    )
    .forEach(
        checkbox => {
            checkbox.addEventListener(
                "change",
                updateValidation
            );
        }
    );

saveButton.addEventListener(
    "click",
    saveConfiguration
);

applicationButton.addEventListener(
    "click",
    startEvaluation
);

resultsButton.addEventListener(
    "click",
    openResults
);

updateKxGroups();
checkClientConfiguration();
listenForEvaluationEvents();