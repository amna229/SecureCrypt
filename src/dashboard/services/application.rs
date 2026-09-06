//! External application process management.
//!
//! This module provides generic process management for applications
//! executed as part of an evaluation.
//!
//! SecureCrypt does not know which concrete application is being
//! executed. The executable name or path is provided by the caller.
//!
//! Processes are started directly through `tokio::process::Command`
//! without invoking a shell.

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;

use tokio::process::{Child, Command};
use tokio::sync::Mutex;

/// Manages processes belonging to an external application.
///
/// The service keeps every started process in a registry so that all
/// processes can be stopped when the evaluation finishes.
#[derive(Clone)]
pub struct ApplicationService {
    processes: Arc<Mutex<Vec<Child>>>,
}

impl ApplicationService {
    /// Creates an empty application process manager.
    pub fn new() -> Self {
        Self {
            processes: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Starts an external application process.
    ///
    /// The executable is provided by the caller and is not interpreted
    /// through a shell. Arguments and environment variables are passed
    /// directly to the operating system process.
    ///
    /// This keeps the service independent of any particular application.
    pub async fn start(
        &self,
        program: &str,
        args: &[String],
        environment: &HashMap<String, String>,
    ) -> Result<(), String> {
        if program.trim().is_empty() {
            return Err("Application executable cannot be empty".to_string());
        }

        let mut command = Command::new(program);

        command
            .args(args)
            .envs(environment)
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let child = command.spawn().map_err(|error| {
            format!(
                "Failed to start external application '{}': {}",
                program, error
            )
        })?;

        println!(
            "External application '{}' started with PID {:?}",
            program,
            child.id()
        );

        let mut processes = self.processes.lock().await;
        processes.push(child);

        Ok(())
    }

    /// Stops every process started by this service.
    ///
    /// All registered processes are attempted even if one of them
    /// fails to terminate.
    pub async fn stop(&self) -> Result<(), String> {
        let mut processes = self.processes.lock().await;
        let mut errors = Vec::new();

        while let Some(mut process) = processes.pop() {
            if let Err(error) = process.kill().await {
                errors.push(error.to_string());
            }
        }

        if errors.is_empty() {
            println!("External application processes stopped");
            Ok(())
        } else {
            Err(format!(
                "Some external application processes could not be stopped: {}",
                errors.join("; ")
            ))
        }
    }
}
