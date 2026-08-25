//! Resource monitoring service.
//!
//! This module provides the functionality required to monitor CPU and memory
//! usage of the Docker containers involved in an evaluation.
//!
//! The implementation is divided into separate modules for container
//! information, metric accumulation, monitoring, and CPU calculation.

pub mod accumulator;
pub mod cpu;
pub mod monitor;

pub use accumulator::MonitoredContainer;
pub use monitor::ResourceMonitor;
