const params = new URLSearchParams(window.location.search);

const transferId = params.get("transfer_id");

const classicalRadio = document.querySelector('input[value="classical"]');

const postQuantumRadio = document.querySelector('input[value="post_quantum"]');

const classicalKx = document.getElementById("classical-kx");

const postQuantumKx = document.getElementById("post-quantum-kx");

const cipherCheckboxes = document.querySelectorAll('input[name="cipher_suites"]');

const saveButton = document.getElementById("save-button");

const cipherError = document.getElementById("cipher-error");

const kxError = document.getElementById("kx-error");


function getActiveKxContainer() {

    return classicalRadio.checked
        ? classicalKx
        : postQuantumKx;

}


function getSelectedValues(container, selector) {
    return Array.from(
        container.querySelectorAll(`${selector}:checked`)
    ).map(
        checkbox => checkbox.value
    );
}


function updateKxGroups() {
    const isClassical =
        classicalRadio.checked;

    classicalKx.style.display =
        isClassical ? "block" : "none";

    postQuantumKx.style.display =
        isClassical ? "none" : "block";

    updateValidation();
}


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
        noKxSelected;
}


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
            )
    };
}


async function saveConfiguration() {

    const config = getConfiguration();

    const response = await fetch(
        "/info/server",
        {
            method: "POST",

            headers: {
                "Content-Type": "application/json"
            },

            body: JSON.stringify(config)
        }
    );

    if (response.ok) {

        console.log("Configuration saved");

        window.location.href = "/client?transfer_id=" + encodeURIComponent(transferId);
    }
}


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