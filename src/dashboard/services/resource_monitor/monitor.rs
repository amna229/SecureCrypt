//! Docker resource monitoring.
//!
//! This module manages the asynchronous collection of CPU and memory
//! statistics from the containers involved in an evaluation.

use crate::dashboard::metrics::resource_repository::ResourceRepository;

use super::accumulator::{MonitoredContainer, ResourceAccumulator};

use super::cpu::calculate_cpu_percent;

use bollard::{Docker, query_parameters::StatsOptionsBuilder};

use chrono::Utc;

use futures_util::StreamExt;

use std::collections::HashMap;

use tokio::task::JoinHandle;

use tokio_util::sync::CancellationToken;

use uuid::Uuid;

/// Handles resource monitoring for the containers involved in
/// an evaluation.
///
/// The monitor runs asynchronously in a Tokio task and can be stopped
/// through a cancellation token. Once stopped, the accumulated metrics
/// are persisted in the database.
pub struct ResourceMonitor {
    cancel_token: CancellationToken,
    handle: JoinHandle<()>,
}

impl ResourceMonitor {
    /// Starts resource monitoring for the specified Docker containers.
    ///
    /// A statistics stream is created for every monitored container.
    /// CPU and memory measurements are collected while the evaluation
    /// is running.
    ///
    /// When the cancellation token is triggered, the monitoring task
    /// stops collecting samples and persists the accumulated metrics.
    pub fn start(
        docker: Docker,
        db: sqlx::PgPool,
        evaluation_id: Uuid,
        containers: Vec<MonitoredContainer>,
        parent_token: CancellationToken,
    ) -> Self {
        let cancel_token = parent_token.child_token();

        let task_token = cancel_token.clone();

        let handle = tokio::spawn(async move {
            let mut accumulators: HashMap<String, ResourceAccumulator> = HashMap::new();

            use futures_util::stream::SelectAll;

            let mut stats_streams = SelectAll::new();

            // Create a Docker statistics stream
            // for every monitored container.
            for container in &containers {
                let options = StatsOptionsBuilder::default()
                    .stream(true)
                    .one_shot(false)
                    .build();

                let stream = docker.stats(&container.name, Some(options)).map({
                    let container = container.clone();

                    move |result| (container.clone(), result)
                });

                stats_streams.push(stream);
            }

            // Collect resource samples until monitoring
            // is cancelled or all streams have ended.
            loop {
                tokio::select! {
                    _ = task_token.cancelled() => {
                        break;
                    }

                    Some((container, result)) =
                        stats_streams.next() =>
                    {
                        let stats = match result {
                            Ok(stats) => stats,

                            Err(error) => {
                                eprintln!(
                                    "Error getting resource \
                                     stats for {}: {}",
                                    container.name,
                                    error
                                );

                                continue;
                            }
                        };

                        let cpu_percent =
                            match calculate_cpu_percent(
                                &stats
                            ) {
                                Some(value) => value,
                                None => continue,
                            };

                        let memory_bytes = stats
                            .memory_stats
                            .as_ref()
                            .and_then(|memory| memory.usage)
                            .unwrap_or(0);

                        let accumulator =
                            accumulators
                                .entry(
                                    container.name.clone()
                                )
                                .or_insert_with(|| {
                                    ResourceAccumulator::new(
                                        container.role.clone(),
                                        container.client_id,
                                    )
                                });

                        accumulator.add_sample(
                            cpu_percent,
                            memory_bytes,
                        );
                    }

                    else => {
                        break;
                    }
                }
            }

            // Persist the accumulated metrics after
            // monitoring stops.
            let repository = ResourceRepository::new(db);

            let timestamp = Utc::now();

            for (_, accumulator) in accumulators {
                if accumulator.samples() == 0 {
                    continue;
                }

                if let Err(error) = repository
                    .save(
                        Uuid::new_v4(),
                        evaluation_id,
                        &accumulator.role,
                        accumulator.client_id,
                        timestamp,
                        accumulator.cpu_average(),
                        accumulator.cpu_peak(),
                        accumulator.memory_peak() as i64,
                    )
                    .await
                {
                    eprintln!("Error saving resource metrics: {}", error);
                }
            }

            println!(
                "Resource monitoring finished \
                 for evaluation {}",
                evaluation_id
            );
        });

        Self {
            cancel_token,
            handle,
        }
    }

    /// Stops resource monitoring and waits for the
    /// monitoring task to finish.
    ///
    /// Cancellation is requested first, allowing the
    /// background task to finish its persistence phase
    /// before returning.
    pub async fn stop(self) {
        self.cancel_token.cancel();

        if let Err(error) = self.handle.await {
            eprintln!("Resource monitoring task failed: {}", error);
        }
    }
}
