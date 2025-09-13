use crate::banner::{BannerApi, BannerApiError};
use crate::data::models::ScrapeJob;
use crate::error::Result;
use crate::scraper::jobs::JobType;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{debug, error, info, trace, warn};

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
    pub async fn run(&self) {
        info!(worker_id = self.id, "Worker started.");
        loop {
            match self.fetch_and_lock_job().await {
                Ok(Some(job)) => {
                    let job_id = job.id;
                    debug!(worker_id = self.id, job_id = job.id, "Processing job");
                    if let Err(e) = self.process_job(job).await {
                        // Check if the error is due to an invalid session
                        if let Some(BannerApiError::InvalidSession(_)) =
                            e.downcast_ref::<BannerApiError>()
                        {
                            warn!(
                                worker_id = self.id,
                                job_id, "Invalid session detected. Forcing session refresh."
                            );
                        } else {
                            error!(worker_id = self.id, job_id, error = ?e, "Failed to process job");
                        }

                        // Unlock the job so it can be retried
                        if let Err(unlock_err) = self.unlock_job(job_id).await {
                            error!(
                                worker_id = self.id,
                                job_id,
                                ?unlock_err,
                                "Failed to unlock job"
                            );
                        }
                    } else {
                        debug!(worker_id = self.id, job_id, "Job completed");
                        // If successful, delete the job.
                        if let Err(delete_err) = self.delete_job(job_id).await {
                            error!(
                                worker_id = self.id,
                                job_id,
                                ?delete_err,
                                "Failed to delete job"
                            );
                        }
                    }
                }
                Ok(None) => {
                    // No job found, wait for a bit before polling again.
                    trace!(worker_id = self.id, "No jobs available, waiting");
                    time::sleep(Duration::from_secs(5)).await;
                }
                Err(e) => {
                    warn!(worker_id = self.id, error = ?e, "Failed to fetch job");
                    // Wait before retrying to avoid spamming errors.
                    time::sleep(Duration::from_secs(10)).await;
                }
            }
        }
    }

    /// Atomically fetches a job from the queue, locking it for processing.
    ///
    /// This uses a `FOR UPDATE SKIP LOCKED` query to ensure that multiple
    /// workers can poll the queue concurrently without conflicts.
    async fn fetch_and_lock_job(&self) -> Result<Option<ScrapeJob>> {
        let mut tx = self.db_pool.begin().await?;

        let job = sqlx::query_as::<_, ScrapeJob>(
            "SELECT * FROM scrape_jobs WHERE locked_at IS NULL AND execute_at <= NOW() ORDER BY priority DESC, execute_at ASC LIMIT 1 FOR UPDATE SKIP LOCKED"
        )
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(ref job) = job {
            sqlx::query("UPDATE scrape_jobs SET locked_at = NOW() WHERE id = $1")
                .bind(job.id)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        Ok(job)
    }

    async fn process_job(&self, job: ScrapeJob) -> Result<()> {
        // Convert the database job to our job type
        let job_type = JobType::from_target_type_and_payload(
            job.target_type,
            job.target_payload,
        )?;
        
        // Get the job implementation
        let job_impl = job_type.as_job();
        
        debug!(
            worker_id = self.id,
            job_id = job.id,
            description = job_impl.description(),
            "Processing job"
        );
        
        // Process the job
        job_impl.process(&self.banner_api, &self.db_pool).await
    }



    async fn delete_job(&self, job_id: i32) -> Result<()> {
        sqlx::query("DELETE FROM scrape_jobs WHERE id = $1")
            .bind(job_id)
            .execute(&self.db_pool)
            .await?;
        Ok(())
    }

    async fn unlock_job(&self, job_id: i32) -> Result<()> {
        sqlx::query("UPDATE scrape_jobs SET locked_at = NULL WHERE id = $1")
            .bind(job_id)
            .execute(&self.db_pool)
            .await?;
        info!(worker_id = self.id, job_id, "Job unlocked for retry");
        Ok(())
    }
}
