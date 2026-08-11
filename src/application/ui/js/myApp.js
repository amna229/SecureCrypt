const startButton =
    document.getElementById("start-transfer");

const status =
    document.getElementById("status");

const fileSize =
    document.getElementById("file-size");

const fileSizeUnit =
    document.getElementById("file-size-unit");

const numFiles =
    document.getElementById("num-files");


function normalizeInputs() {

    if (fileSize.value < 1) {

        fileSize.value = 1;

    }

    if (numFiles.value < 1) {

        numFiles.value = 1;

    }

}


function getConfiguration() {

    return {

        operation:
            document.querySelector(
                'input[name="operation"]:checked'
            ).value,

        file_size:
            Number(fileSize.value),

        file_size_unit:
            fileSizeUnit.value,

        num_files:
            Number(numFiles.value)

    };

}


async function startEvaluation() {

    normalizeInputs();

    const configuration =
        getConfiguration();

    console.log(
        "Application configuration:",
        configuration
    );

    status.textContent =
        "Starting evaluation...";

    await fetch("/application/start", {
        
        method: "POST",

        headers: {"Content-Type": "application/json"},

        body: JSON.stringify(configuration)

    }
);

}


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
    startEvaluation
);