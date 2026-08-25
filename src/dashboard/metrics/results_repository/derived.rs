//! Derived evaluation metrics.
//!
//! This module calculates the relative performance changes between
//! classical and post-quantum evaluation results.

use super::{DerivedMetrics, HandshakeSummary, ResourceSummary, TransferSummary};

/// Creates an empty set of derived metrics.
fn empty_metrics() -> DerivedMetrics {
    DerivedMetrics {
        handshake_overhead_percent: None,
        transfer_overhead_percent: None,
        cpu_overhead_percent: None,
        memory_overhead_percent: None,
        throughput_change_percent: None,
    }
}

/// Calculates derived metrics for an experimental group.
///
/// Metrics are calculated only when both classical and post-quantum
/// evaluations are available.
pub(crate) fn calculate_derived(group: &super::EvaluationGroup) -> DerivedMetrics {
    let has_classical = group
        .evaluations
        .iter()
        .any(|evaluation| evaluation.crypto_mode == "classical");

    let has_post_quantum = group
        .evaluations
        .iter()
        .any(|evaluation| evaluation.crypto_mode == "post_quantum");

    if !has_classical || !has_post_quantum {
        return empty_metrics();
    }

    let classical_handshake = weighted_mean(&group.handshakes, "classical", |value| value.mean_ms);

    let pqc_handshake = weighted_mean(&group.handshakes, "post_quantum", |value| value.mean_ms);

    let classical_transfer = weighted_mean(&group.transfers, "classical", |value| {
        value.mean_duration_ms
    });

    let pqc_transfer = weighted_mean(&group.transfers, "post_quantum", |value| {
        value.mean_duration_ms
    });

    let classical_throughput = weighted_mean(&group.transfers, "classical", |value| {
        value.mean_throughput_mbps
    });

    let pqc_throughput = weighted_mean(&group.transfers, "post_quantum", |value| {
        value.mean_throughput_mbps
    });

    let classical_cpu = weighted_resource_mean(&group.resources, "classical", "client", |value| {
        value.mean_cpu_avg_percent
    });

    let pqc_cpu = weighted_resource_mean(&group.resources, "post_quantum", "client", |value| {
        value.mean_cpu_avg_percent
    });

    let classical_memory =
        weighted_resource_mean(&group.resources, "classical", "client", |value| {
            value.mean_memory_peak_bytes
        });

    let pqc_memory = weighted_resource_mean(&group.resources, "post_quantum", "client", |value| {
        value.mean_memory_peak_bytes
    });

    DerivedMetrics {
        handshake_overhead_percent: percentage_change(classical_handshake, pqc_handshake),

        transfer_overhead_percent: percentage_change(classical_transfer, pqc_transfer),

        cpu_overhead_percent: percentage_change(classical_cpu, pqc_cpu),

        memory_overhead_percent: percentage_change(classical_memory, pqc_memory),

        throughput_change_percent: percentage_change(classical_throughput, pqc_throughput),
    }
}

/// Calculates a sample-weighted mean for a collection of summaries.
fn weighted_mean<T>(values: &[T], mode: &str, value: impl Fn(&T) -> f64) -> Option<f64>
where
    T: HasSamples,
{
    let selected = values
        .iter()
        .filter(|item| item.mode() == mode)
        .collect::<Vec<_>>();

    let total_samples = selected.iter().map(|item| item.samples()).sum::<i64>();

    if total_samples <= 0 {
        return None;
    }

    Some(
        selected
            .iter()
            .map(|item| value(*item) * item.samples() as f64)
            .sum::<f64>()
            / total_samples as f64,
    )
}

/// Calculates a sample-weighted mean for resource summaries
/// filtered by cryptographic mode and container role.
fn weighted_resource_mean(
    values: &[ResourceSummary],
    mode: &str,
    role: &str,
    value: impl Fn(&ResourceSummary) -> f64,
) -> Option<f64> {
    let selected = values
        .iter()
        .filter(|item| item.crypto_mode == mode && item.role == role)
        .collect::<Vec<_>>();

    let total_samples = selected.iter().map(|item| item.samples).sum::<i64>();

    if total_samples <= 0 {
        return None;
    }

    Some(
        selected
            .iter()
            .map(|item| value(*item) * item.samples as f64)
            .sum::<f64>()
            / total_samples as f64,
    )
}

/// Calculates the percentage change between two values.
///
/// The classical value is used as the baseline. A `None` result is
/// returned when either value is unavailable or the baseline is zero.
fn percentage_change(classical: Option<f64>, post_quantum: Option<f64>) -> Option<f64> {
    let classical = classical?;
    let post_quantum = post_quantum?;

    if classical == 0.0 {
        return None;
    }

    Some(((post_quantum - classical) / classical) * 100.0)
}

/// Provides access to the cryptographic mode and sample count
/// of aggregated result summaries.
pub(crate) trait HasSamples {
    fn mode(&self) -> &str;
    fn samples(&self) -> i64;
}

impl HasSamples for HandshakeSummary {
    fn mode(&self) -> &str {
        &self.crypto_mode
    }

    fn samples(&self) -> i64 {
        self.samples
    }
}

impl HasSamples for TransferSummary {
    fn mode(&self) -> &str {
        &self.crypto_mode
    }

    fn samples(&self) -> i64 {
        self.samples
    }
}
