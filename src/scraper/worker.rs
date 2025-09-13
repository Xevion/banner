use crate::banner::{BannerApi, BannerApiError, Course, SearchQuery, Term};
use crate::data::models::ScrapeJob;
use crate::error::Result;
use serde_json::Value;
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
        match job.target_type {
            crate::data::models::TargetType::Subject => {
                self.process_subject_job(&job.target_payload).await
            }
            _ => {
                warn!(worker_id = self.id, job_id = job.id, "unhandled job type");
                Ok(())
            }
        }
    }

    async fn process_subject_job(&self, payload: &Value) -> Result<()> {
        let subject_code = payload["subject"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid subject payload"))?;
        info!(
            worker_id = self.id,
            subject = subject_code,
            "Scraping subject"
        );

        let term = Term::get_current().inner().to_string();
        let query = SearchQuery::new().subject(subject_code).max_results(500);

        let search_result = self
            .banner_api
            .search(&term, &query, "subjectDescription", false)
            .await?;

        if let Some(courses_from_api) = search_result.data {
            info!(
                worker_id = self.id,
                subject = subject_code,
                count = courses_from_api.len(),
                "Found courses"
            );
            for course in courses_from_api {
                self.upsert_course(&course).await?;
            }
        }

        Ok(())
    }

    async fn upsert_course(&self, course: &Course) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO courses (crn, subject, course_number, title, term_code, enrollment, max_enrollment, wait_count, wait_capacity, last_scraped_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (crn, term_code) DO UPDATE SET
                subject = EXCLUDED.subject,
                course_number = EXCLUDED.course_number,
                title = EXCLUDED.title,
                enrollment = EXCLUDED.enrollment,
                max_enrollment = EXCLUDED.max_enrollment,
                wait_count = EXCLUDED.wait_count,
                wait_capacity = EXCLUDED.wait_capacity,
                last_scraped_at = EXCLUDED.last_scraped_at
            "#,
        )
        .bind(&course.course_reference_number)
        .bind(&course.subject)
        .bind(&course.course_number)
        .bind(&course.course_title)
        .bind(&course.term)
        .bind(course.enrollment)
        .bind(course.maximum_enrollment)
        .bind(course.wait_count)
        .bind(course.wait_capacity)
        .bind(chrono::Utc::now())
        .execute(&self.db_pool)
        .await?;

        Ok(())
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
