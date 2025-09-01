use crate::banner::{BannerApi, Term};
use crate::data::models::{ScrapePriority, TargetType};
use crate::error::Result;
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

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
        info!("Scheduler service started.");
        let mut interval = time::interval(Duration::from_secs(60)); // Runs every minute

        loop {
            interval.tick().await;
            info!("Scheduler waking up to analyze and schedule jobs...");
            if let Err(e) = self.schedule_jobs().await {
                error!(error = ?e, "Failed to schedule jobs");
            }
        }
    }

    /// The core logic for deciding what jobs to create.
    async fn schedule_jobs(&self) -> Result<()> {
        // For now, we will implement a simple baseline scheduling strategy:
        // 1. Get a list of all subjects from the Banner API.
        // 2. For each subject, check if an active (not locked, not completed) job already exists.
        // 3. If no job exists, create a new, low-priority job to be executed in the near future.
        let term = Term::get_current().inner().to_string();

        info!(
            term = term,
            "[Scheduler] Enqueuing baseline subject scrape jobs..."
        );

        let subjects = self.banner_api.get_subjects("", &term, 1, 500).await?;

        for subject in subjects {
            let payload = json!({ "subject": subject.code });

            let existing_job: Option<(i32,)> = sqlx::query_as(
                "SELECT id FROM scrape_jobs WHERE target_type = $1 AND target_payload = $2 AND locked_at IS NULL"
            )
            .bind(TargetType::Subject)
            .bind(&payload)
            .fetch_optional(&self.db_pool)
            .await?;

            if existing_job.is_some() {
                continue;
            }

            sqlx::query(
                "INSERT INTO scrape_jobs (target_type, target_payload, priority, execute_at) VALUES ($1, $2, $3, $4)"
            )
            .bind(TargetType::Subject)
            .bind(&payload)
            .bind(ScrapePriority::Low)
            .bind(chrono::Utc::now())
            .execute(&self.db_pool)
            .await?;

            info!(subject = subject.code, "[Scheduler] Enqueued new job");
        }

        info!("[Scheduler] Job scheduling complete.");
        Ok(())
    }
}
