use crate::banner::{BannerApi, Term};
use crate::data::models::{ScrapePriority, TargetType};
use crate::data::scrape_jobs;
use crate::error::Result;
use crate::scraper::jobs::subject::SubjectJob;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

/// Periodically analyzes data and enqueues prioritized scrape jobs.
pub struct Scheduler {
    db_pool: PgPool,
    banner_api: Arc<BannerApi>,
}

impl Scheduler {
    pub fn new(db_pool: PgPool, banner_api: Arc<BannerApi>) -> Self {
        Self {
            db_pool,
            banner_api,
        }
    }

    /// Runs the scheduler's main loop with graceful shutdown support.
    ///
    /// The scheduler wakes up every 60 seconds to analyze data and enqueue jobs.
    /// When a shutdown signal is received:
    /// 1. Any in-progress scheduling work is gracefully cancelled via CancellationToken
    /// 2. The scheduler waits up to 5 seconds for work to complete
    /// 3. If timeout occurs, the task is abandoned (it will be aborted when dropped)
    ///
    /// This ensures that shutdown is responsive even if scheduling work is blocked.
    pub async fn run(&self, mut shutdown_rx: broadcast::Receiver<()>) {
        info!("Scheduler service started");

        let work_interval = Duration::from_secs(60);
        let mut next_run = time::Instant::now();
        let mut current_work: Option<(tokio::task::JoinHandle<()>, CancellationToken)> = None;

        loop {
            tokio::select! {
                _ = time::sleep_until(next_run) => {
                    let cancel_token = CancellationToken::new();

                    // Spawn work in separate task to allow graceful cancellation during shutdown.
                    // Without this, shutdown would have to wait for the full scheduling cycle.
                    let work_handle = tokio::spawn({
                        let db_pool = self.db_pool.clone();
                        let banner_api = self.banner_api.clone();
                        let cancel_token = cancel_token.clone();

                        async move {
                            tokio::select! {
                                result = Self::schedule_jobs_impl(&db_pool, &banner_api) => {
                                    if let Err(e) = result {
                                        error!(error = ?e, "Failed to schedule jobs");
                                    }
                                }
                                _ = cancel_token.cancelled() => {
                                    debug!("Scheduling work cancelled gracefully");
                                }
                            }
                        }
                    });

                    current_work = Some((work_handle, cancel_token));
                    next_run = time::Instant::now() + work_interval;
                }
                _ = shutdown_rx.recv() => {
                    info!("Scheduler received shutdown signal");

                    if let Some((handle, cancel_token)) = current_work.take() {
                        cancel_token.cancel();

                        // Wait briefly for graceful completion
                        if tokio::time::timeout(Duration::from_secs(5), handle).await.is_err() {
                            warn!("Scheduling work did not complete within 5s, abandoning");
                        } else {
                            debug!("Scheduling work completed gracefully");
                        }
                    }

                    info!("Scheduler exiting gracefully");
                    break;
                }
            }
        }
    }

    /// Core scheduling logic that analyzes data and creates scrape jobs.
    ///
    /// Strategy:
    /// 1. Fetch all subjects for the current term from Banner API
    /// 2. Query existing jobs in a single batch query
    /// 3. Create jobs only for subjects that don't have pending jobs
    ///
    /// This is a static method (not &self) to allow it to be called from spawned tasks.
    #[tracing::instrument(skip_all, fields(term))]
    async fn schedule_jobs_impl(db_pool: &PgPool, banner_api: &BannerApi) -> Result<()> {
        // For now, we will implement a simple baseline scheduling strategy:
        // 1. Get a list of all subjects from the Banner API.
        // 2. Query existing jobs for all subjects in a single query.
        // 3. Create new jobs only for subjects that don't have existing jobs.
        let term = Term::get_current().inner().to_string();

        tracing::Span::current().record("term", term.as_str());
        debug!(term = term, "Enqueuing subject jobs");

        let subjects = banner_api.get_subjects("", &term, 1, 500).await?;
        debug!(
            subject_count = subjects.len(),
            "Retrieved subjects from API"
        );

        // Create payloads for all subjects
        let subject_payloads: Vec<_> = subjects
            .iter()
            .map(|subject| json!({ "subject": subject.code }))
            .collect();

        // Query existing jobs for all subjects in a single query
        let existing_payloads = scrape_jobs::find_existing_job_payloads(
            TargetType::Subject,
            &subject_payloads,
            db_pool,
        )
        .await?;

        // Filter out subjects that already have jobs and prepare new jobs
        let mut skipped_count = 0;
        let new_jobs: Vec<_> = subjects
            .into_iter()
            .filter_map(|subject| {
                let job = SubjectJob::new(subject.code.clone());
                let payload = serde_json::to_value(&job).unwrap();
                let payload_str = payload.to_string();

                if existing_payloads.contains(&payload_str) {
                    skipped_count += 1;
                    None
                } else {
                    Some((payload, subject.code))
                }
            })
            .collect();

        if skipped_count > 0 {
            debug!(count = skipped_count, "Skipped subjects with existing jobs");
        }

        // Insert all new jobs in a single batch
        if !new_jobs.is_empty() {
            for (_, subject_code) in &new_jobs {
                debug!(subject = subject_code, "New job enqueued for subject");
            }

            let jobs: Vec<_> = new_jobs
                .into_iter()
                .map(|(payload, _)| (payload, TargetType::Subject, ScrapePriority::Low))
                .collect();

            scrape_jobs::batch_insert_jobs(&jobs, db_pool).await?;
        }

        debug!("Job scheduling complete");
        Ok(())
    }
}
