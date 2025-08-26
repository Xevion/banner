use tokio::sync::broadcast;
use tracing::{error, info, warn};

pub mod bot;
pub mod dummy;
pub mod manager;

#[derive(Debug)]
pub enum ServiceResult {
    GracefulShutdown,
    NormalCompletion,
    Error(anyhow::Error),
}

/// Common trait for all services in the application
#[async_trait::async_trait]
pub trait Service: Send + Sync {
    /// The name of the service for logging
    fn name(&self) -> &'static str;

    /// Run the service's main work loop
    async fn run(&mut self) -> Result<(), anyhow::Error>;

    /// Gracefully shutdown the service
    async fn shutdown(&mut self) -> Result<(), anyhow::Error>;
}

/// Generic service runner that handles the lifecycle
pub async fn run_service(
    mut service: Box<dyn Service>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> ServiceResult {
    let name = service.name();
    info!(service = name, "Service started");

    let work = async {
        match service.run().await {
            Ok(()) => {
                warn!(service = name, "Service completed unexpectedly");
                ServiceResult::NormalCompletion
            }
            Err(e) => {
                error!(service = name, "Service failed: {e}");
                ServiceResult::Error(e)
            }
        }
    };

    tokio::select! {
        result = work => result,
        _ = shutdown_rx.recv() => {
            info!(service = name, "Shutting down...");
            let start_time = std::time::Instant::now();

            match service.shutdown().await {
                Ok(()) => {
                    let elapsed = start_time.elapsed();
                    info!(service = name, "Shutdown completed in {elapsed:.2?}");
                    ServiceResult::GracefulShutdown
                }
                Err(e) => {
                    let elapsed = start_time.elapsed();
                    error!(service = name, "Shutdown failed after {elapsed:.2?}: {e}");
                    ServiceResult::Error(e)
                }
            }
        }
    }
}
