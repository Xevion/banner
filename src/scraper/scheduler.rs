use crate::banner::{BannerApi, Term};
use crate::data::models::{ScrapePriority, TargetType};
use crate::error::Result;
use crate::scraper::jobs::subject::SubjectJob;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{debug, error, info, trace};

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

    /// Runs the scheduler's main loop.
    pub async fn run(&self) {
        info!("Scheduler service started");
        let mut interval = time::interval(Duration::from_secs(60)); // Runs every minute

        loop {
            interval.tick().await;
            // Scheduler analyzing data...
            if let Err(e) = self.schedule_jobs().await {
                error!(error = ?e, "Failed to schedule jobs");
            }
        }
    }

    /// The core logic for deciding what jobs to create.
    async fn schedule_jobs(&self) -> Result<()> {
        // For now, we will implement a simple baseline scheduling strategy:
        // 1. Get a list of all subjects from the Banner API.
        // 2. Query existing jobs for all subjects in a single query.
        // 3. Create new jobs only for subjects that don't have existing jobs.
        let term = Term::get_current().inner().to_string();

        debug!(term = term, "Enqueuing subject jobs");

        let subjects = self.banner_api.get_subjects("", &term, 1, 500).await?;
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
        let existing_jobs: Vec<(serde_json::Value,)> = sqlx::query_as(
            "SELECT target_payload FROM scrape_jobs 
             WHERE target_type = $1 AND target_payload = ANY($2) AND locked_at IS NULL",
        )
        .bind(TargetType::Subject)
        .bind(&subject_payloads)
        .fetch_all(&self.db_pool)
        .await?;

        // Convert to a HashSet for efficient lookup
        let existing_payloads: std::collections::HashSet<String> = existing_jobs
            .into_iter()
            .map(|(payload,)| payload.to_string())
            .collect();

        // Filter out subjects that already have jobs and prepare new jobs
        let new_jobs: Vec<_> = subjects
            .into_iter()
            .filter_map(|subject| {
                let job = SubjectJob::new(subject.code.clone());
                let payload = serde_json::to_value(&job).unwrap();
                let payload_str = payload.to_string();

                if existing_payloads.contains(&payload_str) {
                    trace!(subject = subject.code, "Job already exists, skipping");
                    None
                } else {
                    Some((payload, subject.code))
                }
            })
            .collect();

        // Insert all new jobs in a single batch
        if !new_jobs.is_empty() {
            let now = chrono::Utc::now();
            let mut tx = self.db_pool.begin().await?;

            for (payload, subject_code) in new_jobs {
                sqlx::query(
                    "INSERT INTO scrape_jobs (target_type, target_payload, priority, execute_at) VALUES ($1, $2, $3, $4)"
                )
                .bind(TargetType::Subject)
                .bind(&payload)
                .bind(ScrapePriority::Low)
                .bind(now)
                .execute(&mut *tx)
                .await?;

                debug!(subject = subject_code, "New job enqueued for subject");
            }

            tx.commit().await?;
        }

        debug!("Job scheduling complete");
        Ok(())
    }
}
