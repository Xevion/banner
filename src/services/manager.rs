use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use crate::services::{Service, ServiceResult, run_service};

/// Manages multiple services and their lifecycle
pub struct ServiceManager {
    registered_services: HashMap<String, Box<dyn Service>>,
    running_services: HashMap<String, JoinHandle<ServiceResult>>,
    shutdown_tx: broadcast::Sender<()>,
}

impl ServiceManager {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            registered_services: HashMap::new(),
            running_services: HashMap::new(),
            shutdown_tx,
        }
    }

    /// Register a service to be managed (not yet spawned)
    pub fn register_service(&mut self, name: &str, service: Box<dyn Service>) {
        self.registered_services.insert(name.to_string(), service);
    }

    /// Spawn all registered services
    pub fn spawn_all(&mut self) {
        for (name, service) in self.registered_services.drain() {
            let shutdown_rx = self.shutdown_tx.subscribe();
            let handle = tokio::spawn(run_service(service, shutdown_rx));
            self.running_services.insert(name, handle);
        }
        info!("Spawned {} services", self.running_services.len());
    }

    /// Run all services until one completes or fails
    /// Returns the first service that completes and its result
    pub async fn run(&mut self) -> (String, ServiceResult) {
        if self.running_services.is_empty() {
            return (
                "none".to_string(),
                ServiceResult::Error(anyhow::anyhow!("No services to run")),
            );
        }

        info!(
            "ServiceManager running {} services",
            self.running_services.len()
        );

        // Wait for any service to complete
        loop {
            let mut completed_services = Vec::new();

            for (name, handle) in &mut self.running_services {
                if handle.is_finished() {
                    completed_services.push(name.clone());
                }
            }

            if let Some(completed_name) = completed_services.first() {
                let handle = self.running_services.remove(completed_name).unwrap();
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
        if self.running_services.is_empty() {
            info!("No services to shutdown");
            return Ok(());
        }

        info!(
            "Shutting down {} services with {}s timeout",
            self.running_services.len(),
            timeout.as_secs()
        );

        // Signal all services to shutdown
        let _ = self.shutdown_tx.send(());

        // Wait for all services to complete with timeout
        let shutdown_result = tokio::time::timeout(timeout, async {
            let mut completed = Vec::new();
            let mut failed = Vec::new();

            while !self.running_services.is_empty() {
                let mut to_remove = Vec::new();

                for (name, handle) in &mut self.running_services {
                    if handle.is_finished() {
                        to_remove.push(name.clone());
                    }
                }

                for name in to_remove {
                    let handle = self.running_services.remove(&name).unwrap();
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

                if !self.running_services.is_empty() {
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
                let pending_services: Vec<String> = self.running_services.keys().cloned().collect();
                Err(pending_services)
            }
        }
    }
}
