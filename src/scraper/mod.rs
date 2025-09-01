pub mod scheduler;
pub mod worker;

use crate::banner::BannerApi;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::info;

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
}

impl ScraperService {
    /// Creates a new `ScraperService`.
    pub fn new(db_pool: PgPool, banner_api: Arc<BannerApi>) -> Self {
        Self {
            db_pool,
            banner_api,
            scheduler_handle: None,
            worker_handles: Vec::new(),
        }
    }

    /// Starts the scheduler and a pool of workers.
    pub fn start(&mut self) {
        info!("ScraperService starting...");

        let scheduler = Scheduler::new(self.db_pool.clone(), self.banner_api.clone());
        let scheduler_handle = tokio::spawn(async move {
            scheduler.run().await;
        });
        self.scheduler_handle = Some(scheduler_handle);
        info!("Scheduler task spawned.");

        let worker_count = 4; // This could be configurable
        for i in 0..worker_count {
            let worker = Worker::new(i, self.db_pool.clone(), self.banner_api.clone());
            let worker_handle = tokio::spawn(async move {
                worker.run().await;
            });
            self.worker_handles.push(worker_handle);
        }
        info!("Spawned {} worker tasks.", self.worker_handles.len());
    }

    /// Signals all child tasks to gracefully shut down.
    pub async fn shutdown(&mut self) {
        info!("Shutting down scraper service...");
        if let Some(handle) = self.scheduler_handle.take() {
            handle.abort();
        }
        for handle in self.worker_handles.drain(..) {
            handle.abort();
        }
        info!("Scraper service shutdown.");
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
        self.shutdown().await;
        Ok(())
    }
}
