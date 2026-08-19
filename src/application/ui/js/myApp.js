const startButton =vdocument.getElementById("start-transfer");

const status = document.getElementById("status");

const fileSize = document.getElementById("file-size");

const fileSizeUnit = document.getElementById("file-size-unit");

const numFiles = document.getElementById("num-files");

const resultsButton = document.getElementById("results-button");

const stopEvaluationButton = document.getElementById("stop-evaluation");

const reconfigureButton = document.getElementById("reconfigure-button");

let evaluationStarted = false;

resultsButton.disabled = false;



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

        file_size: Number(fileSize.value),

        file_size_unit: fileSizeUnit.value,

        num_files: Number(numFiles.value)

    };

}



async function startEvaluation() {

    normalizeInputs();


    const configuration = getConfiguration();


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

        stopEvaluationButton.disabled = false;

        reconfigureButton.disabled = true;

        resultsButton.disabled = false;

        status.textContent = "Evaluation running";

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
                    JSON.stringify(
                        configuration
                    )
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



async function stopEvaluation() {

    status.textContent = "Stopping evaluation...";


    const response =
        await fetch(
            "http://127.0.0.1:3000/eval/stop",
            {
                method: "POST"
            }
        );


    if (!response.ok) {

        status.textContent = "Error stopping evaluation";

        return;
    }

    evaluationStarted = false;

    stopEvaluationButton.disabled = true;

    reconfigureButton.disabled = false;

    resultsButton.disabled = false;

    status.textContent = "Evaluation stopped";

}



async function reconfigure() {

    const response =
        await fetch(
            "http://127.0.0.1:3000/eval/reset",
            {
                method: "POST"
            }
        );


    if (!response.ok) {

        status.textContent = "Error resetting configuration";

        return;
    }

    evaluationStarted = false;

    stopEvaluationButton.disabled = true;

    reconfigureButton.disabled = true;

    resultsButton.disabled = false;

    document.querySelector('input[name="operation"][value="upload"]').checked = true;

    fileSize.value = 1;

    fileSizeUnit.value = "KB";

    numFiles.value = 1;

    status.textContent = "Ready to configure a new evaluation";

    window.location.href = "http://127.0.0.1:3000/server";

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
    async () => {

        if (evaluationStarted) {

            status.textContent = "Stopping evaluation...";

            const response =
                await fetch(
                    "http://127.0.0.1:3000/eval/stop",
                    {
                        method: "POST"
                    }
                );


            if (!response.ok) {

                status.textContent = "Error stopping evaluation";

                return;
            }


            evaluationStarted = false;
        }


        window.location.href = "/results";

    }
);



stopEvaluationButton.addEventListener(
    "click",
    stopEvaluation
);



reconfigureButton.addEventListener(
    "click",
    reconfigure
);