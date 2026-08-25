//! Evaluation event management.
//!
//! This module provides the functionality required to publish
//! evaluation lifecycle events to dashboard clients.

use crate::dashboard::services::manager::Manager;
use crate::dashboard::state::EvaluationEvent;

use uuid::Uuid;

impl Manager {
    /// Publishes an evaluation lifecycle event to subscribed dashboard clients.
    ///
    /// The event contains the evaluation identifier, the affected client
    /// when applicable, and the current progress.
    pub(crate) fn send_evaluation_event(
        &self,
        event_type: &str,
        evaluation_id: Uuid,
        client_id: Option<i32>,
        completed: i64,
        total: i64,
    ) {
        let event = EvaluationEvent {
            event_type: event_type.to_string(),
            evaluation_id,
            client_id,
            completed,
            total,
        };

        let _ = self.state.evaluation_events.send(event);
    }
}
