//! Resource metric accumulation.
//!
//! This module defines the data structures used to identify monitored
//! containers and accumulate their resource usage during an evaluation.

/// Describes a Docker container whose resource usage is monitored.
///
/// The role identifies the purpose of the container within the evaluation,
/// while `client_id` identifies an individual client container when applicable.
#[derive(Debug, Clone)]
pub struct MonitoredContainer {
    pub name: String,
    pub role: String,
    pub client_id: Option<i32>,
}

/// Accumulates resource measurements for a monitored container.
///
/// CPU usage is accumulated to calculate the average and tracked separately
/// to determine the peak CPU usage. Memory usage is tracked using the maximum
/// observed value.
#[derive(Debug)]
pub struct ResourceAccumulator {
    pub(crate) role: String,
    pub(crate) client_id: Option<i32>,
    cpu_sum: f64,
    cpu_peak: f64,
    memory_peak: u64,
    samples: u64,
}

impl ResourceAccumulator {
    /// Creates an empty resource accumulator for a container.
    pub fn new(role: String, client_id: Option<i32>) -> Self {
        Self {
            role,
            client_id,
            cpu_sum: 0.0,
            cpu_peak: 0.0,
            memory_peak: 0,
            samples: 0,
        }
    }

    /// Adds a new CPU and memory measurement to the accumulator.
    ///
    /// The CPU value contributes to the average and may update the recorded
    /// CPU peak. The memory value updates the maximum observed memory usage.
    pub fn add_sample(&mut self, cpu_percent: f64, memory_bytes: u64) {
        self.cpu_sum += cpu_percent;

        if cpu_percent > self.cpu_peak {
            self.cpu_peak = cpu_percent;
        }

        if memory_bytes > self.memory_peak {
            self.memory_peak = memory_bytes;
        }

        self.samples += 1;
    }

    /// Calculates the average CPU usage across all collected samples.
    pub fn cpu_average(&self) -> f64 {
        if self.samples == 0 {
            0.0
        } else {
            self.cpu_sum / self.samples as f64
        }
    }

    /// Returns the number of collected samples.
    pub fn samples(&self) -> u64 {
        self.samples
    }

    /// Returns the peak CPU usage.
    pub fn cpu_peak(&self) -> f64 {
        self.cpu_peak
    }

    /// Returns the peak memory usage.
    pub fn memory_peak(&self) -> u64 {
        self.memory_peak
    }
}





//Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_empty_accumulator() {
        let accumulator = ResourceAccumulator::new("server".to_string(), None);

        assert_eq!(accumulator.samples(), 0);
        assert_eq!(accumulator.cpu_average(), 0.0);
        assert_eq!(accumulator.cpu_peak(), 0.0);
        assert_eq!(accumulator.memory_peak(), 0);
    }

    #[test]
    fn accumulates_resource_samples() {
        let mut accumulator = ResourceAccumulator::new("server".to_string(), None);

        accumulator.add_sample(10.0, 1000);
        accumulator.add_sample(20.0, 2000);
        accumulator.add_sample(30.0, 1500);

        assert_eq!(accumulator.samples(), 3);
        assert_eq!(accumulator.cpu_average(), 20.0);
        assert_eq!(accumulator.cpu_peak(), 30.0);
        assert_eq!(accumulator.memory_peak(), 2000);
    }

    #[test]
    fn tracks_peak_values_independently() {
        let mut accumulator =
            ResourceAccumulator::new("client".to_string(), Some(1));

        accumulator.add_sample(50.0, 5000);
        accumulator.add_sample(20.0, 8000);
        accumulator.add_sample(40.0, 3000);

        assert_eq!(accumulator.cpu_peak(), 50.0);
        assert_eq!(accumulator.memory_peak(), 8000);
    }

    #[test]
    fn average_cpu_is_zero_without_samples() {
        let accumulator = ResourceAccumulator::new("server".to_string(), None);

        assert_eq!(accumulator.cpu_average(), 0.0);
    }
}