const startButton = document.getElementById("start-transfer");

const status = document.getElementById("status");

const fileSize = document.getElementById("file-size");

const fileSizeUnit = document.getElementById("file-size-unit");

const numFiles = document.getElementById("num-files");

const resultsButton = document.getElementById("results-button");


let evaluationStarted = false;

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

    const configuration = getConfiguration();

    console.log("Application configuration:", configuration);

    if (!evaluationStarted) {

        status.textContent = "Starting evaluation environment...";


        const dashboardResponse =
            await fetch(
                "http://127.0.0.1:3000/eval/start",
                {
                    method: "POST"
                }
            );


        if (!dashboardResponse.ok) {

            status.textContent = "Error starting evaluation";

            return;

        }

        evaluationStarted = true;

        console.log("Evaluation environment started");

    }

    status.textContent = "Saving transfer...";


    const response =
        await fetch(
            "/application/start",
            {
                method: "POST",

                headers: {
                    "Content-Type":
                        "application/json"
                },

                body:
                    JSON.stringify(configuration)
            }
        );


    if (!response.ok) {

        status.textContent = "Error starting transfer";

        return;

    }


    const result = await response.json();


    console.log("Transfer ID:", result.transfer_id);


    status.textContent = "Transfer created";

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


resultsButton.addEventListener(
    "click",
    () => {

        window.location.href =
            "/results";

    }
);