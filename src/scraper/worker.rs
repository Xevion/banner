use crate::banner::{BannerApi, BannerApiError};
use crate::data::models::ScrapeJob;
use crate::data::scrape_jobs;
use crate::error::Result;
use crate::scraper::jobs::{JobError, JobType};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time;
use tracing::{Instrument, debug, error, info, trace, warn};

/// A single worker instance.
///
/// Each worker runs in its own asynchronous task and continuously polls the
/// database for scrape jobs to execute.
pub struct Worker {
    id: usize, // For logging purposes
    db_pool: PgPool,
    banner_api: Arc<BannerApi>,
}

impl Worker {
    pub fn new(id: usize, db_pool: PgPool, banner_api: Arc<BannerApi>) -> Self {
        Self {
            id,
            db_pool,
            banner_api,
        }
    }

    /// Runs the worker's main loop.
    pub async fn run(&self, mut shutdown_rx: broadcast::Receiver<()>) {
        info!(worker_id = self.id, "Worker started");

        loop {
            // Fetch and lock a job, racing against shutdown signal
            let job = tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!(worker_id = self.id, "Worker received shutdown signal, exiting gracefully");
                    break;
                }
                result = self.fetch_and_lock_job() => {
                    match result {
                        Ok(Some(job)) => job,
                        Ok(None) => {
                            trace!(worker_id = self.id, "No jobs available, waiting");
                            time::sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                        Err(e) => {
                            warn!(worker_id = self.id, error = ?e, "Failed to fetch job, waiting");
                            time::sleep(Duration::from_secs(10)).await;
                            continue;
                        }
                    }
                }
            };

            let job_id = job.id;
            let retry_count = job.retry_count;
            let max_retries = job.max_retries;
            let start = std::time::Instant::now();

            // Process the job, racing against shutdown signal
            let process_result = tokio::select! {
                _ = shutdown_rx.recv() => {
                    self.handle_shutdown_during_processing(job_id).await;
                    break;
                }
                result = self.process_job(job) => result
            };

            let duration = start.elapsed();

            // Handle the job processing result
            self.handle_job_result(job_id, retry_count, max_retries, process_result, duration)
                .await;
        }
    }

    /// Atomically fetches a job from the queue, locking it for processing.
    ///
    /// This uses a `FOR UPDATE SKIP LOCKED` query to ensure that multiple
    /// workers can poll the queue concurrently without conflicts.
    async fn fetch_and_lock_job(&self) -> Result<Option<ScrapeJob>> {
        scrape_jobs::fetch_and_lock_job(&self.db_pool).await
    }

    async fn process_job(&self, job: ScrapeJob) -> Result<(), JobError> {
        // Convert the database job to our job type
        let job_type = JobType::from_target_type_and_payload(job.target_type, job.target_payload)
            .map_err(|e| JobError::Unrecoverable(anyhow::anyhow!(e)))?; // Parse errors are unrecoverable

        // Get the job implementation
        let job_impl = job_type.boxed();

        // Create span with job context
        let span = tracing::debug_span!(
            "process_job",
            job_id = job.id,
            job_type = job_impl.description()
        );

        async move {
            debug!(
                worker_id = self.id,
                job_id = job.id,
                description = job_impl.description(),
                "Processing job"
            );

            // Process the job - API errors are recoverable
            job_impl
                .process(&self.banner_api, &self.db_pool)
                .await
                .map_err(JobError::Recoverable)?;

            Ok(())
        }
        .instrument(span)
        .await
    }

    async fn delete_job(&self, job_id: i32) -> Result<()> {
        scrape_jobs::delete_job(job_id, &self.db_pool).await
    }

    async fn unlock_job(&self, job_id: i32) -> Result<()> {
        scrape_jobs::unlock_job(job_id, &self.db_pool).await
    }

    async fn unlock_and_increment_retry(&self, job_id: i32, max_retries: i32) -> Result<bool> {
        scrape_jobs::unlock_and_increment_retry(job_id, max_retries, &self.db_pool).await
    }

    /// Handle shutdown signal received during job processing
    async fn handle_shutdown_during_processing(&self, job_id: i32) {
        info!(
            worker_id = self.id,
            job_id, "Shutdown received during job processing"
        );

        if let Err(e) = self.unlock_job(job_id).await {
            warn!(
                worker_id = self.id,
                job_id,
                error = ?e,
                "Failed to unlock job during shutdown"
            );
        } else {
            debug!(worker_id = self.id, job_id, "Job unlocked during shutdown");
        }

        info!(worker_id = self.id, "Worker exiting gracefully");
    }

    /// Handle the result of job processing
    async fn handle_job_result(
        &self,
        job_id: i32,
        retry_count: i32,
        max_retries: i32,
        result: Result<(), JobError>,
        duration: std::time::Duration,
    ) {
        match result {
            Ok(()) => {
                debug!(
                    worker_id = self.id,
                    job_id,
                    duration_ms = duration.as_millis(),
                    "Job completed successfully"
                );
                if let Err(e) = self.delete_job(job_id).await {
                    error!(worker_id = self.id, job_id, error = ?e, "Failed to delete completed job");
                }
            }
            Err(JobError::Recoverable(e)) => {
                self.handle_recoverable_error(job_id, retry_count, max_retries, e, duration)
                    .await;
            }
            Err(JobError::Unrecoverable(e)) => {
                error!(
                    worker_id = self.id,
                    job_id,
                    duration_ms = duration.as_millis(),
                    error = ?e,
                    "Job corrupted, deleting"
                );
                if let Err(e) = self.delete_job(job_id).await {
                    error!(worker_id = self.id, job_id, error = ?e, "Failed to delete corrupted job");
                }
            }
        }
    }

    /// Handle recoverable errors by logging appropriately and unlocking the job
    async fn handle_recoverable_error(
        &self,
        job_id: i32,
        retry_count: i32,
        max_retries: i32,
        e: anyhow::Error,
        duration: std::time::Duration,
    ) {
        let next_attempt = retry_count.saturating_add(1);
        let remaining_retries = max_retries.saturating_sub(next_attempt);

        // Log the error appropriately based on type
        if let Some(BannerApiError::InvalidSession(_)) = e.downcast_ref::<BannerApiError>() {
            warn!(
                worker_id = self.id,
                job_id,
                duration_ms = duration.as_millis(),
                retry_attempt = next_attempt,
                max_retries = max_retries,
                remaining_retries = remaining_retries,
                "Invalid session detected, will retry"
            );
        } else {
            error!(
                worker_id = self.id,
                job_id,
                duration_ms = duration.as_millis(),
                retry_attempt = next_attempt,
                max_retries = max_retries,
                remaining_retries = remaining_retries,
                error = ?e,
                "Failed to process job, will retry"
            );
        }

        // Atomically unlock and increment retry count, checking if retry is allowed
        match self.unlock_and_increment_retry(job_id, max_retries).await {
            Ok(can_retry) if can_retry => {
                info!(
                    worker_id = self.id,
                    job_id,
                    retry_attempt = next_attempt,
                    remaining_retries = remaining_retries,
                    "Job unlocked for retry"
                );
            }
            Ok(_) => {
                // Max retries exceeded (detected atomically)
                error!(
                    worker_id = self.id,
                    job_id,
                    duration_ms = duration.as_millis(),
                    retry_count = next_attempt,
                    max_retries = max_retries,
                    error = ?e,
                    "Job failed permanently (max retries exceeded), deleting"
                );
                if let Err(e) = self.delete_job(job_id).await {
                    error!(worker_id = self.id, job_id, error = ?e, "Failed to delete failed job");
                }
            }
            Err(e) => {
                error!(worker_id = self.id, job_id, error = ?e, "Failed to unlock and increment retry count");
            }
        }
    }
}
