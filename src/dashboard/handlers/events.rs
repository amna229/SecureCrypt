//! Evaluation event handlers.
//!
//! This module contains the HTTP handlers responsible for receiving
//! evaluation metrics and streaming them to dashboard clients.

use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::sse::{Event, Sse},
};

use chrono::Utc;
use futures_util::stream;
use std::{convert::Infallible, sync::Arc};
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::dashboard::database::storage::DashboardStorage;
use crate::dashboard::metrics::{
    handshake_repository::HandshakeRepository, transfer_repository::TransferRepository,
};
use crate::dashboard::services::manager::Manager;
use crate::dashboard::state::{DashboardEvent, DashboardState};

#[derive(Debug, serde::Deserialize)]
pub struct HandshakeEventRequest {
    pub handshake_id: Uuid,
    pub evaluation_id: Uuid,
    pub client_id: i32,
    pub handshake_duration_ms: Option<i64>,
    pub kx_group: Option<String>,
    pub cipher_suite: Option<String>,
    pub success: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct TransferEventRequest {
    pub event_type: String,
    pub execution_id: String,
    pub transfer_id: Uuid,
    pub evaluation_id: Uuid,
    pub client_id: i32,
    pub operation: String,
    pub file_size: i64,
    pub file_size_unit: String,
    pub num_files: i32,
    pub bytes_transferred: Option<i64>,
    pub duration_ms: Option<i64>,
    pub throughput_mbps: Option<f64>,
    pub crypto_mode: Option<String>,
    pub kx_group: Option<String>,
    pub cipher_suite: Option<String>,
    pub success: bool,
    pub error_type: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ClientFinishedRequest {
    pub evaluation_id: Uuid,
    pub client_id: i32,
}

/// Receives a TLS handshake metric and stores it
/// using the configured dashboard storage backend.
pub async fn handshake_event(
    State(state): State<Arc<DashboardState>>,
    Json(event): Json<HandshakeEventRequest>,
) -> StatusCode {
    let repository = HandshakeRepository::new(state.db.clone());

    match repository
        .save(
            event.handshake_id,
            event.evaluation_id,
            event.client_id,
            event.handshake_duration_ms,
            event.kx_group.as_deref(),
            event.cipher_suite.as_deref(),
            event.success,
        )
        .await
    {
        Ok(()) => StatusCode::OK,

        Err(error) => {
            eprintln!("Error saving handshake metric: {}", error);

            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// Receives notification that an evaluation client has finished.
///
/// When all configured clients for the current evaluation have finished,
/// the evaluation is stopped and cleaned up automatically.
pub async fn client_finished(
    State(state): State<Arc<DashboardState>>,
    Json(event): Json<ClientFinishedRequest>,
) -> StatusCode {
    let current_evaluation_id = {
        let evaluation_id = state.evaluation_id.lock().await;

        *evaluation_id
    };

    if current_evaluation_id != Some(event.evaluation_id) {
        return StatusCode::BAD_REQUEST;
    }

    let expected_clients = {
        let configs = state.client_configs.lock().await;

        configs
            .iter()
            .map(|config| config.num_connections as i32)
            .sum::<i32>()
    };

    let completed = {
        let mut completed_clients = state.completed_clients.lock().await;

        completed_clients.insert((event.evaluation_id, event.client_id));

        completed_clients
            .iter()
            .filter(|(evaluation_id, _)| *evaluation_id == event.evaluation_id)
            .count() as i32
    };

    println!(
        "Client {} finished evaluation {}/{}",
        event.client_id, completed, expected_clients
    );

    if expected_clients > 0 && completed >= expected_clients {
        let state_clone = Arc::clone(&state);

        tokio::spawn(async move {
            let manager = Manager::new(state_clone);

            if let Err(error) = manager.stop().await {
                eprintln!("Error stopping completed evaluation: {}", error);
            }
        });
    }

    StatusCode::OK
}

/// Receives a generic application transfer metric
/// and stores it using the configured dashboard storage backend.
pub async fn transfer_event(
    State(state): State<Arc<DashboardState>>,
    Json(event): Json<TransferEventRequest>,
) -> StatusCode {
    let repository = TransferRepository::new(state.db.clone());

    let created_at = Utc::now();

    let finished_at = if event.success {
        Some(Utc::now())
    } else {
        None
    };

    if let Err(error) = repository
        .save(
            &event.execution_id,
            event.transfer_id,
            event.evaluation_id,
            event.client_id,
            &event.operation,
            event.file_size,
            &event.file_size_unit,
            event.num_files,
            event.bytes_transferred,
            event.duration_ms,
            event.throughput_mbps,
            event.crypto_mode.as_deref(),
            event.kx_group.as_deref(),
            event.cipher_suite.as_deref(),
            Some(event.success),
            event.error_type.as_deref(),
            created_at,
            finished_at,
        )
        .await
    {
        eprintln!("Error saving transfer metric: {}", error);

        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    let (total, completed) = match &state.db {
        DashboardStorage::Postgres(pool) => {
            match sqlx::query_as::<_, (i64, i64)>(
                "SELECT
                    COUNT(*)::BIGINT AS total,
                    COUNT(*) FILTER (
                        WHERE finished_at IS NOT NULL
                    )::BIGINT AS completed
                 FROM transfer
                 WHERE evaluation_id = $1",
            )
            .bind(event.evaluation_id)
            .fetch_one(pool)
            .await
            {
                Ok(values) => values,

                Err(error) => {
                    eprintln!("Error getting transfer progress: {}", error);

                    return StatusCode::INTERNAL_SERVER_ERROR;
                }
            }
        }

        DashboardStorage::Memory(database) => {
            let database = database.lock().await;

            let total = database
                .transfers
                .iter()
                .filter(|transfer| transfer.evaluation_id == event.evaluation_id)
                .count() as i64;

            let completed = database
                .transfers
                .iter()
                .filter(|transfer| {
                    transfer.evaluation_id == event.evaluation_id && transfer.finished_at.is_some()
                })
                .count() as i64;

            (total, completed)
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
