use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::services::ServiceResult;

/// Manages multiple services and their lifecycle
pub struct ServiceManager {
    services: HashMap<String, JoinHandle<ServiceResult>>,
    shutdown_tx: broadcast::Sender<()>,
}

impl ServiceManager {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            services: HashMap::new(),
            shutdown_tx,
        }
    }

    /// Add a service to be managed
    pub fn add_service(&mut self, name: String, handle: JoinHandle<ServiceResult>) {
        self.services.insert(name, handle);
    }

    /// Get a shutdown receiver for services to subscribe to
    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    /// Run all services until one completes or fails
    /// Returns the first service that completes and its result
    pub async fn run(&mut self) -> (String, ServiceResult) {
        if self.services.is_empty() {
            return (
                "none".to_string(),
                ServiceResult::Error(anyhow::anyhow!("No services to run")),
            );
        }

        info!("ServiceManager running {} services", self.services.len());

        // Wait for any service to complete
        loop {
            let mut completed_services = Vec::new();

            for (name, handle) in &mut self.services {
                if handle.is_finished() {
                    completed_services.push(name.clone());
                }
            }

            if let Some(completed_name) = completed_services.first() {
                let handle = self.services.remove(completed_name).unwrap();
                match handle.await {
                    Ok(result) => {
                        return (completed_name.clone(), result);
                    }
                    Err(e) => {
                        error!(service = completed_name, "Service task panicked: {e}");
                        return (
                            completed_name.clone(),
                            ServiceResult::Error(anyhow::anyhow!("Task panic: {e}")),
                        );
                    }
                }
            }

            // Small delay to prevent busy-waiting
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Shutdown all services gracefully with a timeout
    /// Returns Ok(()) if all services shut down, or Err(Vec<String>) with names of services that timed out
    pub async fn shutdown(mut self, timeout: Duration) -> Result<(), Vec<String>> {
        if self.services.is_empty() {
            info!("No services to shutdown");
            return Ok(());
        }

        info!(
            "Shutting down {} services with {}s timeout",
            self.services.len(),
            timeout.as_secs()
        );

        // Signal all services to shutdown
        let _ = self.shutdown_tx.send(());

        // Wait for all services to complete with timeout
        let shutdown_result = tokio::time::timeout(timeout, async {
            let mut completed = Vec::new();
            let mut failed = Vec::new();

            while !self.services.is_empty() {
                let mut to_remove = Vec::new();

                for (name, handle) in &mut self.services {
                    if handle.is_finished() {
                        to_remove.push(name.clone());
                    }
                }

                for name in to_remove {
                    let handle = self.services.remove(&name).unwrap();
                    match handle.await {
                        Ok(ServiceResult::GracefulShutdown) => {
                            completed.push(name);
                        }
                        Ok(ServiceResult::NormalCompletion) => {
                            warn!(service = name, "Service completed normally during shutdown");
                            completed.push(name);
                        }
                        Ok(ServiceResult::Error(e)) => {
                            error!(service = name, "Service error during shutdown: {e}");
                            failed.push(name);
                        }
                        Err(e) => {
                            error!(service = name, "Service panic during shutdown: {e}");
                            failed.push(name);
                        }
                    }
                }

                if !self.services.is_empty() {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }

            (completed, failed)
        })
        .await;

        match shutdown_result {
            Ok((completed, failed)) => {
                if !completed.is_empty() {
                    info!("Services shutdown completed: {}", completed.join(", "));
                }
                if !failed.is_empty() {
                    warn!("Services had errors during shutdown: {}", failed.join(", "));
                }
                Ok(())
            }
            Err(_) => {
                // Timeout occurred - return names of services that didn't complete
                let pending_services: Vec<String> = self.services.keys().cloned().collect();
                Err(pending_services)
            }
        }
    }
}
