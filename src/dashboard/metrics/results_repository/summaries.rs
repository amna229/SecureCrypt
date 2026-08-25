//! Statistical summary construction.
//!
//! This module transforms raw handshake, transfer and resource samples
//! into the aggregated summaries used by the dashboard.

use super::{HandshakeSummary, ResourceSummary, TransferSummary};

/// Builds a statistical summary from TLS handshake measurements.
pub(crate) fn build_handshake_summary(
    crypto_mode: String,
    kx_group: String,
    mut values: Vec<i64>,
) -> HandshakeSummary {
    values.sort_unstable();

    let samples = values.len() as i64;

    if samples == 0 {
        return HandshakeSummary {
            crypto_mode,
            kx_group,
            samples: 0,
            mean_ms: 0.0,
            median_ms: 0.0,
            stddev_ms: 0.0,
            min_ms: 0,
            max_ms: 0,
        };
    }

    let mean_ms = values.iter().map(|value| *value as f64).sum::<f64>() / samples as f64;

    let median_ms = if values.len() % 2 == 0 {
        let right = values.len() / 2;
        let left = right - 1;

        (values[left] as f64 + values[right] as f64) / 2.0
    } else {
        values[values.len() / 2] as f64
    };

    let variance = values
        .iter()
        .map(|value| {
            let difference = *value as f64 - mean_ms;

            difference * difference
        })
        .sum::<f64>()
        / samples as f64;

    HandshakeSummary {
        crypto_mode,
        kx_group,
        samples,
        mean_ms,
        median_ms,
        stddev_ms: variance.sqrt(),
        min_ms: *values.first().unwrap_or(&0),
        max_ms: *values.last().unwrap_or(&0),
    }
}

/// Builds an aggregated summary from transfer measurements.
pub(crate) fn build_transfer_summary(
    crypto_mode: String,
    kx_group: String,
    values: Vec<(i64, i64, f64)>,
) -> TransferSummary {
    let samples = values.len() as i64;

    if samples == 0 {
        return TransferSummary {
            crypto_mode,
            kx_group,
            samples: 0,
            mean_duration_ms: 0.0,
            mean_throughput_mbps: 0.0,
            total_bytes: 0,
        };
    }

    let mean_duration_ms = values
        .iter()
        .map(|(duration, _, _)| *duration as f64)
        .sum::<f64>()
        / samples as f64;

    let mean_throughput_mbps = values
        .iter()
        .map(|(_, _, throughput)| *throughput)
        .sum::<f64>()
        / samples as f64;

    let total_bytes = values.iter().map(|(_, bytes, _)| *bytes).sum::<i64>();

    TransferSummary {
        crypto_mode,
        kx_group,
        samples,
        mean_duration_ms,
        mean_throughput_mbps,
        total_bytes,
    }
}

/// Builds an aggregated summary from container resource measurements.
pub(crate) fn build_resource_summary(
    crypto_mode: String,
    role: String,
    values: Vec<(f64, f64, i64)>,
) -> ResourceSummary {
    let samples = values.len() as i64;

    if samples == 0 {
        return ResourceSummary {
            crypto_mode,
            role,
            samples: 0,
            mean_cpu_avg_percent: 0.0,
            max_cpu_peak_percent: 0.0,
            mean_memory_peak_bytes: 0.0,
            max_memory_peak_bytes: 0,
        };
    }

    let mean_cpu_avg_percent =
        values.iter().map(|(cpu_avg, _, _)| *cpu_avg).sum::<f64>() / samples as f64;

    let max_cpu_peak_percent = values
        .iter()
        .map(|(_, cpu_peak, _)| *cpu_peak)
        .fold(0.0_f64, f64::max);

    let mean_memory_peak_bytes = values
        .iter()
        .map(|(_, _, memory)| *memory as f64)
        .sum::<f64>()
        / samples as f64;

    let max_memory_peak_bytes = values
        .iter()
        .map(|(_, _, memory)| *memory)
        .max()
        .unwrap_or(0);

    ResourceSummary {
        crypto_mode,
        role,
        samples,
        mean_cpu_avg_percent,
        max_cpu_peak_percent,
        mean_memory_peak_bytes,
        max_memory_peak_bytes,
    }
}
