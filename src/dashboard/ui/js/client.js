const classicalRadio =
    document.querySelector('input[value="classical"]');

const postQuantumRadio =
    document.querySelector('input[value="post_quantum"]');

const classicalKx =
    document.getElementById("classical-kx");

const postQuantumKx =
    document.getElementById("post-quantum-kx");

const cipherCheckboxes =
    document.querySelectorAll(
        'input[name="cipher_suites"]'
    );

const numConnections =
    document.getElementById("num-connections");

const saveButton =
    document.getElementById("save-button");

const cipherError =
    document.getElementById("cipher-error");

const kxError =
    document.getElementById("kx-error");


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


function normalizeConnections() {
    if (numConnections.value > 1000) {
        numConnections.value = 1000;
    }

    if (numConnections.value < 1) {
        numConnections.value = 1;
    }
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
            ),

        num_connections:
            Number(numConnections.value)
    };
}


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

    console.log("Respuesta del servidor:", response.status);

    await checkClientConfiguration();

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



async function checkClientConfiguration() {

    const response = await fetch("/info/client/status");


    const configured = await response.json();


    const applicationButton =
        document.getElementById(
            "application-button"
        );


    if(configured){

        applicationButton.disabled = false;

        applicationButton.onclick = async () => {

            applicationButton.disabled = true;

            applicationButton.innerHTML = "Loading...";


            const response =
                await fetch(
                    "/application/start",
                    {
                        method: "POST"
                    }
                );


            if(response.ok){

                window.location.assign(applicationButton.dataset.url);

            }
            else {

                applicationButton.disabled = false;

                applicationButton.innerHTML = "Go to<br>Application";

                alert("Error starting application");

            }

        };

    }

}


updateKxGroups();
checkClientConfiguration();