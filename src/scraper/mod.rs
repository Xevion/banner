pub mod jobs;
pub mod scheduler;
pub mod worker;

use crate::banner::BannerApi;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tracing::{info, warn};

use self::scheduler::Scheduler;
use self::worker::Worker;
use crate::services::Service;

/// The main service that will be managed by the application's `ServiceManager`.
///
/// It holds the shared resources (database pool, API client) and manages the
/// lifecycle of the Scheduler and Worker tasks.
pub struct ScraperService {
    db_pool: PgPool,
    banner_api: Arc<BannerApi>,
    scheduler_handle: Option<JoinHandle<()>>,
    worker_handles: Vec<JoinHandle<()>>,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl ScraperService {
    /// Creates a new `ScraperService`.
    pub fn new(db_pool: PgPool, banner_api: Arc<BannerApi>) -> Self {
        Self {
            db_pool,
            banner_api,
            scheduler_handle: None,
            worker_handles: Vec::new(),
            shutdown_tx: None,
        }
    }

    /// Starts the scheduler and a pool of workers.
    pub fn start(&mut self) {
        info!("ScraperService starting");

        // Create shutdown channel
        let (shutdown_tx, _) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        let scheduler = Scheduler::new(self.db_pool.clone(), self.banner_api.clone());
        let shutdown_rx = shutdown_tx.subscribe();
        let scheduler_handle = tokio::spawn(async move {
            scheduler.run(shutdown_rx).await;
        });
        self.scheduler_handle = Some(scheduler_handle);
        info!("Scheduler task spawned");

        let worker_count = 4; // This could be configurable
        for i in 0..worker_count {
            let worker = Worker::new(i, self.db_pool.clone(), self.banner_api.clone());
            let shutdown_rx = shutdown_tx.subscribe();
            let worker_handle = tokio::spawn(async move {
                worker.run(shutdown_rx).await;
            });
            self.worker_handles.push(worker_handle);
        }
        info!(
            worker_count = self.worker_handles.len(),
            "Spawned worker tasks"
        );
    }

}

#[async_trait::async_trait]
impl Service for ScraperService {
    fn name(&self) -> &'static str {
        "scraper"
    }

    async fn run(&mut self) -> Result<(), anyhow::Error> {
        self.start();
        std::future::pending::<()>().await;
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        info!("Shutting down scraper service");

        // Send shutdown signal to all tasks
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        } else {
            warn!("No shutdown channel found for scraper service");
        }

        // Collect all handles
        let mut all_handles = Vec::new();
        if let Some(handle) = self.scheduler_handle.take() {
            all_handles.push(handle);
        }
        all_handles.append(&mut self.worker_handles);

        // Wait for all tasks to complete with a timeout
        let timeout_duration = Duration::from_secs(5);

        match tokio::time::timeout(
            timeout_duration,
            futures::future::join_all(all_handles),
        )
        .await
        {
            Ok(results) => {
                let failed = results.iter().filter(|r| r.is_err()).count();
                if failed > 0 {
                    warn!(failed_count = failed, "Some scraper tasks failed during shutdown");
                } else {
                    info!("All scraper tasks shutdown gracefully");
                }
            }
            Err(_) => {
                warn!(
                    timeout = format!("{:.2?}", timeout_duration),
                    "Scraper service shutdown timed out"
                );
            }
        }

        Ok(())
    }
}
