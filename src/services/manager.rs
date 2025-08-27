use std::collections::HashMap;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

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
        let service_count = self.registered_services.len();
        let service_names: Vec<_> = self.registered_services.keys().cloned().collect();

        for (name, service) in self.registered_services.drain() {
            let shutdown_rx = self.shutdown_tx.subscribe();
            let handle = tokio::spawn(run_service(service, shutdown_rx));
            self.running_services.insert(name, handle);
        }

        info!(
            service_count,
            services = ?service_names,
            "spawned {} services",
            service_count
        );
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
            "servicemanager running {} services",
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
                        error!(service = completed_name, "service task panicked: {e}");
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

    /// Shutdown all services gracefully with a timeout.
    ///
    /// If any service fails to shutdown, it will return an error containing the names of the services that failed to shutdown.
    /// If all services shutdown successfully, the function will return the duration elapsed.
    pub async fn shutdown(&mut self, timeout: Duration) -> Result<Duration, Vec<String>> {
        let service_count = self.running_services.len();
        let service_names: Vec<_> = self.running_services.keys().cloned().collect();

        info!(
            service_count,
            services = ?service_names,
            timeout = format!("{:.2?}", timeout),
            "shutting down {} services with {:?} timeout",
            service_count,
            timeout
        );

        // Send shutdown signal to all services
        let _ = self.shutdown_tx.send(());

        // Wait for all services to complete
        let start_time = std::time::Instant::now();
        let mut pending_services = Vec::new();

        for (name, handle) in self.running_services.drain() {
            match tokio::time::timeout(timeout, handle).await {
                Ok(Ok(_)) => {
                    debug!(service = name, "service shutdown completed");
                }
                Ok(Err(e)) => {
                    warn!(service = name, error = ?e, "service shutdown failed");
                    pending_services.push(name);
                }
                Err(_) => {
                    warn!(service = name, "service shutdown timed out");
                    pending_services.push(name);
                }
            }
        }

        let elapsed = start_time.elapsed();
        if pending_services.is_empty() {
            info!(
                service_count,
                elapsed = format!("{:.2?}", elapsed),
                "services shutdown completed: {}",
                service_names.join(", ")
            );
            Ok(elapsed)
        } else {
            warn!(
                pending_count = pending_services.len(),
                pending_services = ?pending_services,
                elapsed = format!("{:.2?}", elapsed),
                "services shutdown completed with {} pending: {}",
                pending_services.len(),
                pending_services.join(", ")
            );
            Err(pending_services)
        }
    }
}
