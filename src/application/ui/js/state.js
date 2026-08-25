/**
 * Shared application page state.
 *
 * This module stores the runtime state used by the application
 * page during an evaluation.
 */

export const state = {
    evaluationStarted: false,

    currentOperation: null,

    transferEventSource: null,

    evaluationEventSource: null,

    currentEvaluationId: null,

    evaluationTotalClients: 0,

    redirectToResultsAfterStop: false
};