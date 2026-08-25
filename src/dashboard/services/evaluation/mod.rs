//! Evaluation lifecycle management.
//!
//! This module contains the functionality required to manage
//! evaluation configuration, lifecycle, cleanup, and events.

/// Evaluation lifecycle operations.
///
/// This module is responsible for starting and stopping
/// evaluation environments.
pub mod lifecycle;

/// Evaluation event management.
///
/// This module provides the functionality required to create
/// and publish events describing the current state and progress
/// of an evaluation.
pub mod events;

/// Evaluation configuration management.
///
/// This module provides access to the configured server and
/// clients and allows the evaluation configuration to be reset.
pub mod configuration;

/// Evaluation cleanup operations.
///
/// This module is responsible for removing the resources
/// associated with a completed or failed evaluation.
pub mod cleanup;
