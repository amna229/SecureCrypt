/**
 * Client configuration interface.
 *
 * Handles cryptographic configuration, key exchange selection,
 * connection validation, configuration persistence, and application
 * access once the client configuration has been stored.
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

const numConnections =
    document.getElementById(
        "num-connections"
    );

const saveButton =
    document.getElementById(
        "save-button"
    );

const cipherError =
    document.getElementById(
        "cipher-error"
    );

const kxError =
    document.getElementById(
        "kx-error"
    );

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
 *
 * At least one cipher suite and one key exchange group
 * must be selected, and the number of connections must
 * be within the allowed range.
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
        invalidNumConnections;
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
 * Builds the client configuration object from
 * the current form values.
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
 *
 * When a valid configuration exists, the application
 * navigation button is enabled.
 */
async function checkClientConfiguration() {
    const response =
        await fetch(
            "/info/client/status"
        );

    const configured =
        await response.json();

    const applicationButton =
        document.getElementById(
            "application-button"
        );

    if (configured) {
        applicationButton.disabled =
            false;

        applicationButton.onclick =
            async () => {
                window.location.assign(
                    applicationButton.dataset.url
                );
            };
    }
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

updateKxGroups();
checkClientConfiguration();