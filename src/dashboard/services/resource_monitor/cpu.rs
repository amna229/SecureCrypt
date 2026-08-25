//! CPU usage calculation.
//!
//! This module contains the calculation required to obtain the CPU
//! utilization percentage from Docker container statistics.

/// Calculates the CPU usage percentage from Docker statistics.
///
/// Docker provides the current and previous CPU usage together with
/// system CPU usage. The differences between these measurements are
/// used to calculate the CPU utilization of the container.
pub fn calculate_cpu_percent(stats: &bollard::models::ContainerStatsResponse) -> Option<f64> {
    let cpu_stats = stats.cpu_stats.as_ref()?;
    let previous_cpu_stats = stats.precpu_stats.as_ref()?;

    let current_cpu = cpu_stats.cpu_usage.as_ref()?.total_usage?;

    let previous_cpu = previous_cpu_stats.cpu_usage.as_ref()?.total_usage?;

    let current_system = cpu_stats.system_cpu_usage?;

    let previous_system = previous_cpu_stats.system_cpu_usage?;

    let cpu_delta = current_cpu.saturating_sub(previous_cpu);

    let system_delta = current_system.saturating_sub(previous_system);

    if system_delta == 0 {
        return Some(0.0);
    }

    let num_cpus = cpu_stats.online_cpus.unwrap_or(1) as f64;

    Some((cpu_delta as f64 / system_delta as f64) * num_cpus * 100.0)
}
