//! Evaluation event handlers.
//!
//! This module contains the HTTP handlers responsible for receiving
//! evaluation events and streaming them to dashboard clients.

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::sse::{Event, Sse},
};
use futures_util::stream;
use std::{convert::Infallible, sync::Arc};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::dashboard::state::{DashboardEvent, DashboardState};

#[derive(Debug, serde::Deserialize)]
pub struct TransferEventRequest {
    pub event_type: String,
    pub evaluation_id: Uuid,
    pub execution_id: String,
    pub client_id: i32,
    pub operation: String,
    pub success: bool,
}

/// Receives a transfer event and publishes its progress
/// to subscribed dashboard clients.
pub async fn transfer_event(
    State(state): State<Arc<DashboardState>>,
    Json(event): Json<TransferEventRequest>,
) -> StatusCode {
    let progress = sqlx::query_as::<_, (i64, i64)>(
        "SELECT
                COUNT(*)::BIGINT AS total,
                COUNT(*) FILTER (
                    WHERE finished_at IS NOT NULL
                )::BIGINT AS completed
             FROM transfer
             WHERE evaluation_id = $1",
    )
    .bind(event.evaluation_id)
    .fetch_one(&state.db)
    .await;

    let (total, completed) = match progress {
        Ok(values) => values,

        Err(error) => {
            eprintln!("Error getting transfer progress: {}", error);

            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    };

    let transfer_event = DashboardEvent::Transfer {
        event_type: event.event_type,
        evaluation_id: event.evaluation_id,
        execution_id: event.execution_id,
        client_id: event.client_id,
        operation: event.operation,
        success: event.success,
        completed,
        total,
        all_completed: total > 0 && completed == total,
    };

    let _ = state.transfer_events.send(transfer_event);

    StatusCode::OK
}

/// Streams transfer events to subscribed dashboard clients.
pub async fn transfer_events(
    State(state): State<Arc<DashboardState>>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.transfer_events.subscribe();

    let event_stream = stream::unfold(receiver, |mut receiver| async move {
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    let (event_type, data) = match serde_json::to_string(&event) {
                        Ok(data) => {
                            let event_type = match &event {
                                DashboardEvent::Transfer { event_type, .. } => event_type,

                                DashboardEvent::Evaluation { event_type, .. } => event_type,
                            };

                            (event_type.clone(), data)
                        }

                        Err(error) => {
                            eprintln!("Error serializing dashboard event: {}", error);

                            continue;
                        }
                    };

                    let sse_event = Event::default().event(&event_type).data(data);

                    return Some((Ok(sse_event), receiver));
                }

                Err(broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }

                Err(broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    });

    Sse::new(event_stream)
}

/// Streams evaluation events to subscribed dashboard clients.
pub async fn evaluation_events(
    State(state): State<Arc<DashboardState>>,
) -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let receiver = state.evaluation_events.subscribe();

    let event_stream = stream::unfold(receiver, |mut receiver| async move {
        loop {
            match receiver.recv().await {
                Ok(event) => {
                    let data = match serde_json::to_string(&event) {
                        Ok(data) => data,

                        Err(error) => {
                            eprintln!("Error serializing evaluation event: {}", error);

                            continue;
                        }
                    };

                    let sse_event = Event::default().event(&event.event_type).data(data);

                    return Some((Ok(sse_event), receiver));
                }

                Err(broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }

                Err(broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    });

    Sse::new(event_stream)
}
