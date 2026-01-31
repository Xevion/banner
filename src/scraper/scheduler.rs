use crate::banner::{BannerApi, Term};
use crate::data::models::{ReferenceData, ScrapePriority, TargetType};
use crate::data::scrape_jobs;
use crate::error::Result;
use crate::rmp::RmpClient;
use crate::scraper::adaptive::{SubjectSchedule, SubjectStats, evaluate_subject};
use crate::scraper::jobs::subject::SubjectJob;
use crate::state::ReferenceCache;
use crate::web::ws::{ScrapeJobDto, ScrapeJobEvent};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, broadcast};
use tokio::time;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

/// How often reference data is re-scraped (6 hours).
const REFERENCE_DATA_INTERVAL: Duration = Duration::from_secs(6 * 60 * 60);

/// How often RMP data is synced (24 hours).
const RMP_SYNC_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

/// Periodically analyzes data and enqueues prioritized scrape jobs.
pub struct Scheduler {
    db_pool: PgPool,
    banner_api: Arc<BannerApi>,
    reference_cache: Arc<RwLock<ReferenceCache>>,
    job_events_tx: broadcast::Sender<ScrapeJobEvent>,
}

impl Scheduler {
    pub fn new(
        db_pool: PgPool,
        banner_api: Arc<BannerApi>,
        reference_cache: Arc<RwLock<ReferenceCache>>,
        job_events_tx: broadcast::Sender<ScrapeJobEvent>,
    ) -> Self {
        Self {
            db_pool,
            banner_api,
            reference_cache,
            job_events_tx,
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
        // Scrape reference data immediately on first cycle
        let mut last_ref_scrape = Instant::now() - REFERENCE_DATA_INTERVAL;
        // Sync RMP data immediately on first cycle
        let mut last_rmp_sync = Instant::now() - RMP_SYNC_INTERVAL;

        loop {
            tokio::select! {
                _ = time::sleep_until(next_run) => {
                    let cancel_token = CancellationToken::new();

                    let should_scrape_ref = last_ref_scrape.elapsed() >= REFERENCE_DATA_INTERVAL;
                    let should_sync_rmp = last_rmp_sync.elapsed() >= RMP_SYNC_INTERVAL;

                    // Spawn work in separate task to allow graceful cancellation during shutdown.
                    let work_handle = tokio::spawn({
                        let db_pool = self.db_pool.clone();
                        let banner_api = self.banner_api.clone();
                        let cancel_token = cancel_token.clone();
                        let reference_cache = self.reference_cache.clone();
                        let job_events_tx = self.job_events_tx.clone();

                                async move {
                                    tokio::select! {
                                        _ = async {
                                            // RMP sync is independent of Banner API — run it
                                            // concurrently with reference data scraping so it
                                            // doesn't wait behind rate-limited Banner calls.
                                            let rmp_fut = async {
                                                if should_sync_rmp
                                                    && let Err(e) = Self::sync_rmp_data(&db_pool).await
                                                {
                                                    error!(error = ?e, "Failed to sync RMP data");
                                                }
                                            };

                                            let ref_fut = async {
                                                if should_scrape_ref
                                                    && let Err(e) = Self::scrape_reference_data(&db_pool, &banner_api, &reference_cache).await
                                                {
                                                    error!(error = ?e, "Failed to scrape reference data");
                                                }
                                            };

                                            tokio::join!(rmp_fut, ref_fut);

                                            if let Err(e) = Self::schedule_jobs_impl(&db_pool, &banner_api, Some(&job_events_tx)).await {
                                                error!(error = ?e, "Failed to schedule jobs");
                                            }
                                        } => {}
                                        _ = cancel_token.cancelled() => {
                                            debug!("Scheduling work cancelled gracefully");
                                        }
                                    }
                                }
                    });

                    if should_scrape_ref {
                        last_ref_scrape = Instant::now();
                    }
                    if should_sync_rmp {
                        last_rmp_sync = Instant::now();
                    }

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
    /// Uses adaptive scheduling to determine per-subject scrape intervals based
    /// on recent change rates, failure patterns, and time of day. Only subjects
    /// that are eligible (i.e. their cooldown has elapsed) are enqueued.
    ///
    /// This is a static method (not &self) to allow it to be called from spawned tasks.
    #[tracing::instrument(skip_all, fields(term))]
    async fn schedule_jobs_impl(
        db_pool: &PgPool,
        banner_api: &BannerApi,
        job_events_tx: Option<&broadcast::Sender<ScrapeJobEvent>>,
    ) -> Result<()> {
        let term = Term::get_current().inner().to_string();

        tracing::Span::current().record("term", term.as_str());
        debug!(term = term, "Enqueuing subject jobs");

        let subjects = banner_api.get_subjects("", &term, 1, 500).await?;
        debug!(
            subject_count = subjects.len(),
            "Retrieved subjects from API"
        );

        // Fetch per-subject stats and build a lookup map
        let stats_rows = scrape_jobs::fetch_subject_stats(db_pool).await?;
        let stats_map: HashMap<String, SubjectStats> = stats_rows
            .into_iter()
            .map(|row| {
                let subject = row.subject.clone();
                (subject, SubjectStats::from(row))
            })
            .collect();

        // Evaluate each subject using adaptive scheduling
        let now = Utc::now();
        let is_past_term = false; // Scheduler currently only fetches current term subjects
        let mut eligible_subjects: Vec<String> = Vec::new();
        let mut cooldown_count: usize = 0;
        let mut paused_count: usize = 0;
        let mut read_only_count: usize = 0;

        for subject in &subjects {
            let stats = stats_map.get(&subject.code).cloned().unwrap_or_else(|| {
                // Cold start: no history for this subject
                SubjectStats {
                    subject: subject.code.clone(),
                    recent_runs: 0,
                    avg_change_ratio: 0.0,
                    consecutive_zero_changes: 0,
                    consecutive_empty_fetches: 0,
                    recent_failure_count: 0,
                    recent_success_count: 0,
                    last_completed: DateTime::<Utc>::MIN_UTC,
                }
            });

            match evaluate_subject(&stats, now, is_past_term) {
                SubjectSchedule::Eligible(_) => {
                    eligible_subjects.push(subject.code.clone());
                }
                SubjectSchedule::Cooldown(_) => cooldown_count += 1,
                SubjectSchedule::Paused => paused_count += 1,
                SubjectSchedule::ReadOnly => read_only_count += 1,
            }
        }

        info!(
            total = subjects.len(),
            eligible = eligible_subjects.len(),
            cooldown = cooldown_count,
            paused = paused_count,
            read_only = read_only_count,
            "Adaptive scheduling decisions"
        );

        if eligible_subjects.is_empty() {
            debug!("No eligible subjects to schedule");
            return Ok(());
        }

        // Create payloads only for eligible subjects
        let subject_payloads: Vec<_> = eligible_subjects
            .iter()
            .map(|code| json!({ "subject": code }))
            .collect();

        // Query existing jobs for eligible subjects only
        let existing_payloads = scrape_jobs::find_existing_job_payloads(
            TargetType::Subject,
            &subject_payloads,
            db_pool,
        )
        .await?;

        // Filter out subjects that already have pending jobs
        let mut skipped_count = 0;
        let new_jobs: Vec<_> = eligible_subjects
            .into_iter()
            .filter_map(|subject_code| {
                let job = SubjectJob::new(subject_code.clone());
                let payload = serde_json::to_value(&job).unwrap();
                let payload_str = payload.to_string();

                if existing_payloads.contains(&payload_str) {
                    skipped_count += 1;
                    None
                } else {
                    Some((payload, subject_code))
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

            let inserted = scrape_jobs::batch_insert_jobs(&jobs, db_pool).await?;

            if let Some(tx) = job_events_tx {
                inserted.iter().for_each(|job| {
                    debug!(job_id = job.id, "Emitting JobCreated event");
                    let _ = tx.send(ScrapeJobEvent::JobCreated {
                        job: ScrapeJobDto::from(job),
                    });
                });
            }
        }

        debug!("Job scheduling complete");
        Ok(())
    }

    /// Fetch all RMP professors, upsert to DB, and auto-match against Banner instructors.
    #[tracing::instrument(skip_all)]
    async fn sync_rmp_data(db_pool: &PgPool) -> Result<()> {
        info!("Starting RMP data sync");

        let client = RmpClient::new();
        let professors = client.fetch_all_professors().await?;
        let total = professors.len();

        crate::data::rmp::batch_upsert_rmp_professors(&professors, db_pool).await?;
        info!(total, "RMP professors upserted");

        let stats = crate::data::rmp_matching::generate_candidates(db_pool).await?;
        info!(
            total,
            stats.total_unmatched,
            stats.candidates_created,
            stats.candidates_rescored,
            stats.auto_matched,
            stats.skipped_unparseable,
            stats.skipped_no_candidates,
            "RMP sync complete"
        );

        Ok(())
    }

    /// Scrape all reference data categories from Banner and upsert to DB, then refresh cache.
    #[tracing::instrument(skip_all)]
    async fn scrape_reference_data(
        db_pool: &PgPool,
        banner_api: &BannerApi,
        reference_cache: &Arc<RwLock<ReferenceCache>>,
    ) -> Result<()> {
        let term = Term::get_current().inner().to_string();
        info!(term = %term, "Scraping reference data");

        let mut all_entries = Vec::new();

        // Terms (fetched via session pool, no active session needed)
        match banner_api.sessions.get_terms("", 1, 500).await {
            Ok(terms) => {
                debug!(count = terms.len(), "Fetched terms");
                all_entries.extend(terms.into_iter().map(|t| ReferenceData {
                    category: "term".to_string(),
                    code: t.code,
                    description: t.description,
                }));
            }
            Err(e) => warn!(error = ?e, "Failed to fetch terms"),
        }

        // Subjects
        match banner_api.get_subjects("", &term, 1, 500).await {
            Ok(pairs) => {
                debug!(count = pairs.len(), "Fetched subjects");
                all_entries.extend(pairs.into_iter().map(|p| ReferenceData {
                    category: "subject".to_string(),
                    code: p.code,
                    description: p.description,
                }));
            }
            Err(e) => warn!(error = ?e, "Failed to fetch subjects"),
        }

        // Campuses
        match banner_api.get_campuses(&term).await {
            Ok(pairs) => {
                debug!(count = pairs.len(), "Fetched campuses");
                all_entries.extend(pairs.into_iter().map(|p| ReferenceData {
                    category: "campus".to_string(),
                    code: p.code,
                    description: p.description,
                }));
            }
            Err(e) => warn!(error = ?e, "Failed to fetch campuses"),
        }

        // Instructional methods
        match banner_api.get_instructional_methods(&term).await {
            Ok(pairs) => {
                debug!(count = pairs.len(), "Fetched instructional methods");
                all_entries.extend(pairs.into_iter().map(|p| ReferenceData {
                    category: "instructional_method".to_string(),
                    code: p.code,
                    description: p.description,
                }));
            }
            Err(e) => warn!(error = ?e, "Failed to fetch instructional methods"),
        }

        // Parts of term
        match banner_api.get_parts_of_term(&term).await {
            Ok(pairs) => {
                debug!(count = pairs.len(), "Fetched parts of term");
                all_entries.extend(pairs.into_iter().map(|p| ReferenceData {
                    category: "part_of_term".to_string(),
                    code: p.code,
                    description: p.description,
                }));
            }
            Err(e) => warn!(error = ?e, "Failed to fetch parts of term"),
        }

        // Attributes
        match banner_api.get_attributes(&term).await {
            Ok(pairs) => {
                debug!(count = pairs.len(), "Fetched attributes");
                all_entries.extend(pairs.into_iter().map(|p| ReferenceData {
                    category: "attribute".to_string(),
                    code: p.code,
                    description: p.description,
                }));
            }
            Err(e) => warn!(error = ?e, "Failed to fetch attributes"),
        }

        // Batch upsert all entries
        let total = all_entries.len();
        crate::data::reference::batch_upsert(&all_entries, db_pool).await?;
        info!(total_entries = total, "Reference data upserted to DB");

        // Refresh in-memory cache
        let all = crate::data::reference::get_all(db_pool).await?;
        let count = all.len();
        *reference_cache.write().await = ReferenceCache::from_entries(all);
        info!(entries = count, "Reference cache refreshed");

        Ok(())
    }
}
