//! Database operations for scrape job queue management.

use crate::data::models::{ScrapeJob, ScrapePriority, TargetType};
use crate::error::Result;
use sqlx::PgPool;
use std::collections::HashSet;

/// Atomically fetch and lock the next available scrape job.
///
/// Uses `FOR UPDATE SKIP LOCKED` to allow multiple workers to poll the queue
/// concurrently without conflicts. Only jobs that are unlocked and ready to
/// execute (based on `execute_at`) are considered.
///
/// # Arguments
/// * `db_pool` - PostgreSQL connection pool
///
/// # Returns
/// * `Ok(Some(job))` if a job was successfully fetched and locked
/// * `Ok(None)` if no jobs are available
pub async fn fetch_and_lock_job(db_pool: &PgPool) -> Result<Option<ScrapeJob>> {
    let mut tx = db_pool.begin().await?;

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

/// Delete a scrape job by ID.
///
/// Typically called after a job has been successfully processed or permanently failed.
///
/// # Arguments
/// * `job_id` - The database ID of the job to delete
/// * `db_pool` - PostgreSQL connection pool
pub async fn delete_job(job_id: i32, db_pool: &PgPool) -> Result<()> {
    sqlx::query("DELETE FROM scrape_jobs WHERE id = $1")
        .bind(job_id)
        .execute(db_pool)
        .await?;
    Ok(())
}

/// Unlock a scrape job by clearing its `locked_at` timestamp.
///
/// Used to release a job back to the queue, e.g. during graceful shutdown.
///
/// # Arguments
/// * `job_id` - The database ID of the job to unlock
/// * `db_pool` - PostgreSQL connection pool
pub async fn unlock_job(job_id: i32, db_pool: &PgPool) -> Result<()> {
    sqlx::query("UPDATE scrape_jobs SET locked_at = NULL WHERE id = $1")
        .bind(job_id)
        .execute(db_pool)
        .await?;
    Ok(())
}

/// Atomically unlock a job and increment its retry count.
///
/// Returns whether the job still has retries remaining. This is determined
/// atomically in the database to avoid race conditions between workers.
///
/// # Arguments
/// * `job_id` - The database ID of the job
/// * `max_retries` - Maximum number of retries allowed for this job
/// * `db_pool` - PostgreSQL connection pool
///
/// # Returns
/// * `Ok(true)` if the job was unlocked and retries remain
/// * `Ok(false)` if the job has exhausted its retries
pub async fn unlock_and_increment_retry(
    job_id: i32,
    max_retries: i32,
    db_pool: &PgPool,
) -> Result<bool> {
    let result = sqlx::query_scalar::<_, Option<i32>>(
        "UPDATE scrape_jobs
         SET locked_at = NULL, retry_count = retry_count + 1
         WHERE id = $1
         RETURNING CASE WHEN retry_count < $2 THEN retry_count ELSE NULL END",
    )
    .bind(job_id)
    .bind(max_retries)
    .fetch_one(db_pool)
    .await?;

    Ok(result.is_some())
}

/// Find existing unlocked job payloads matching the given target type and candidates.
///
/// Returns a set of stringified JSON payloads that already exist in the queue,
/// used for deduplication when scheduling new jobs.
///
/// # Arguments
/// * `target_type` - The target type to filter by
/// * `candidate_payloads` - Candidate payloads to check against existing jobs
/// * `db_pool` - PostgreSQL connection pool
///
/// # Returns
/// A `HashSet` of stringified JSON payloads that already have pending jobs
pub async fn find_existing_job_payloads(
    target_type: TargetType,
    candidate_payloads: &[serde_json::Value],
    db_pool: &PgPool,
) -> Result<HashSet<String>> {
    let existing_jobs: Vec<(serde_json::Value,)> = sqlx::query_as(
        "SELECT target_payload FROM scrape_jobs
         WHERE target_type = $1 AND target_payload = ANY($2) AND locked_at IS NULL",
    )
    .bind(target_type)
    .bind(candidate_payloads)
    .fetch_all(db_pool)
    .await?;

    let existing_payloads = existing_jobs
        .into_iter()
        .map(|(payload,)| payload.to_string())
        .collect();

    Ok(existing_payloads)
}

/// Batch insert scrape jobs in a single transaction.
///
/// All jobs are inserted with `execute_at` set to the current time.
///
/// # Arguments
/// * `jobs` - Slice of `(payload, target_type, priority)` tuples to insert
/// * `db_pool` - PostgreSQL connection pool
pub async fn batch_insert_jobs(
    jobs: &[(serde_json::Value, TargetType, ScrapePriority)],
    db_pool: &PgPool,
) -> Result<()> {
    if jobs.is_empty() {
        return Ok(());
    }

    let now = chrono::Utc::now();
    let mut tx = db_pool.begin().await?;

    for (payload, target_type, priority) in jobs {
        sqlx::query(
            "INSERT INTO scrape_jobs (target_type, target_payload, priority, execute_at) VALUES ($1, $2, $3, $4)"
        )
        .bind(target_type)
        .bind(payload)
        .bind(priority)
        .bind(now)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(())
}
