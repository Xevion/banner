#[allow(dead_code)]
mod helpers;

use banner::data::models::{ScrapePriority, TargetType};
use banner::data::scrape_jobs;
use serde_json::json;
use sqlx::PgPool;

// ── fetch_and_lock_job ──────────────────────────────────────────────

#[sqlx::test]
async fn fetch_and_lock_empty_queue(pool: PgPool) {
    let result = scrape_jobs::fetch_and_lock_job(&pool).await.unwrap();
    assert!(result.is_none());
}

#[sqlx::test]
async fn fetch_and_lock_returns_job_and_sets_locked_at(pool: PgPool) {
    let id = helpers::insert_scrape_job(
        &pool,
        TargetType::Subject,
        json!({"subject": "CS"}),
        ScrapePriority::Medium,
        false,
        0,
        3,
    )
    .await;

    let job = scrape_jobs::fetch_and_lock_job(&pool)
        .await
        .unwrap()
        .expect("should return a job");

    assert_eq!(job.id, id);
    assert!(matches!(job.target_type, TargetType::Subject));
    assert_eq!(job.target_payload, json!({"subject": "CS"}));

    // Verify locked_at was set in the database
    let (locked_at,): (Option<chrono::DateTime<chrono::Utc>>,) =
        sqlx::query_as("SELECT locked_at FROM scrape_jobs WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(locked_at.is_some(), "locked_at should be set after fetch");
}

#[sqlx::test]
async fn fetch_and_lock_skips_locked_jobs(pool: PgPool) {
    helpers::insert_scrape_job(
        &pool,
        TargetType::Subject,
        json!({"subject": "CS"}),
        ScrapePriority::Medium,
        true, // locked
        0,
        3,
    )
    .await;

    let result = scrape_jobs::fetch_and_lock_job(&pool).await.unwrap();
    assert!(result.is_none(), "locked jobs should be skipped");
}

#[sqlx::test]
async fn fetch_and_lock_skips_future_execute_at(pool: PgPool) {
    // Insert a job with execute_at in the future via raw SQL
    sqlx::query(
        "INSERT INTO scrape_jobs (target_type, target_payload, priority, execute_at)
         VALUES ('Subject', '{\"subject\": \"CS\"}', 'Medium', NOW() + INTERVAL '1 hour')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = scrape_jobs::fetch_and_lock_job(&pool).await.unwrap();
    assert!(result.is_none(), "future execute_at jobs should be skipped");
}

#[sqlx::test]
async fn fetch_and_lock_priority_desc_ordering(pool: PgPool) {
    // Insert low priority first, then critical
    helpers::insert_scrape_job(
        &pool,
        TargetType::Subject,
        json!({"subject": "LOW"}),
        ScrapePriority::Low,
        false,
        0,
        3,
    )
    .await;

    helpers::insert_scrape_job(
        &pool,
        TargetType::Subject,
        json!({"subject": "CRIT"}),
        ScrapePriority::Critical,
        false,
        0,
        3,
    )
    .await;

    let job = scrape_jobs::fetch_and_lock_job(&pool)
        .await
        .unwrap()
        .expect("should return a job");

    assert_eq!(
        job.target_payload,
        json!({"subject": "CRIT"}),
        "Critical priority should be fetched before Low"
    );
}

#[sqlx::test]
async fn fetch_and_lock_execute_at_asc_ordering(pool: PgPool) {
    // Insert an older job and a newer job, both same priority
    sqlx::query(
        "INSERT INTO scrape_jobs (target_type, target_payload, priority, execute_at)
         VALUES ('Subject', '{\"subject\": \"OLDER\"}', 'Medium', NOW() - INTERVAL '2 hours')",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO scrape_jobs (target_type, target_payload, priority, execute_at)
         VALUES ('Subject', '{\"subject\": \"NEWER\"}', 'Medium', NOW() - INTERVAL '1 hour')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let job = scrape_jobs::fetch_and_lock_job(&pool)
        .await
        .unwrap()
        .expect("should return a job");

    assert_eq!(
        job.target_payload,
        json!({"subject": "OLDER"}),
        "Older execute_at should be fetched first"
    );
}

// ── delete_job ──────────────────────────────────────────────────────

#[sqlx::test]
async fn delete_job_removes_row(pool: PgPool) {
    let id = helpers::insert_scrape_job(
        &pool,
        TargetType::SingleCrn,
        json!({"crn": "12345"}),
        ScrapePriority::High,
        false,
        0,
        3,
    )
    .await;

    scrape_jobs::delete_job(id, &pool).await.unwrap();

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM scrape_jobs WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0, "row should be deleted");
}

#[sqlx::test]
async fn delete_job_nonexistent_id_no_error(pool: PgPool) {
    // Deleting a non-existent ID should not error
    scrape_jobs::delete_job(999_999, &pool).await.unwrap();
}

// ── unlock_job ──────────────────────────────────────────────────────

#[sqlx::test]
async fn unlock_job_clears_locked_at(pool: PgPool) {
    let id = helpers::insert_scrape_job(
        &pool,
        TargetType::CrnList,
        json!({"crns": [1, 2, 3]}),
        ScrapePriority::Medium,
        true, // locked
        0,
        3,
    )
    .await;

    scrape_jobs::unlock_job(id, &pool).await.unwrap();

    let (locked_at,): (Option<chrono::DateTime<chrono::Utc>>,) =
        sqlx::query_as("SELECT locked_at FROM scrape_jobs WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(locked_at.is_none(), "locked_at should be cleared");
}

// ── unlock_and_increment_retry ──────────────────────────────────────

#[sqlx::test]
async fn unlock_and_increment_retry_has_retries_remaining(pool: PgPool) {
    let id = helpers::insert_scrape_job(
        &pool,
        TargetType::Subject,
        json!({"subject": "CS"}),
        ScrapePriority::Medium,
        true,
        0, // retry_count
        3, // max_retries
    )
    .await;

    let result = scrape_jobs::unlock_and_increment_retry(id, 3, &pool)
        .await
        .unwrap();
    assert!(
        result.is_some(),
        "should have retries remaining (0→1, max=3)"
    );

    // Verify state in DB
    let (retry_count, locked_at): (i32, Option<chrono::DateTime<chrono::Utc>>) =
        sqlx::query_as("SELECT retry_count, locked_at FROM scrape_jobs WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(retry_count, 1);
    assert!(locked_at.is_none(), "should be unlocked");
}

#[sqlx::test]
async fn unlock_and_increment_retry_exhausted(pool: PgPool) {
    let id = helpers::insert_scrape_job(
        &pool,
        TargetType::Subject,
        json!({"subject": "CS"}),
        ScrapePriority::Medium,
        true,
        3, // retry_count (already used all 3 retries)
        3, // max_retries
    )
    .await;

    let result = scrape_jobs::unlock_and_increment_retry(id, 3, &pool)
        .await
        .unwrap();
    assert!(
        result.is_none(),
        "should NOT have retries remaining (3→4, max=3)"
    );

    let (retry_count,): (i32,) =
        sqlx::query_as("SELECT retry_count FROM scrape_jobs WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(retry_count, 4);
}

#[sqlx::test]
async fn unlock_and_increment_retry_already_exceeded(pool: PgPool) {
    let id = helpers::insert_scrape_job(
        &pool,
        TargetType::Subject,
        json!({"subject": "CS"}),
        ScrapePriority::Medium,
        true,
        5, // retry_count already past max
        3, // max_retries
    )
    .await;

    let result = scrape_jobs::unlock_and_increment_retry(id, 3, &pool)
        .await
        .unwrap();
    assert!(
        result.is_none(),
        "should NOT have retries remaining (5→6, max=3)"
    );

    let (retry_count,): (i32,) =
        sqlx::query_as("SELECT retry_count FROM scrape_jobs WHERE id = $1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(retry_count, 6);
}

// ── find_existing_job_payloads ──────────────────────────────────────

#[sqlx::test]
async fn find_existing_payloads_returns_matching(pool: PgPool) {
    let payload_a = json!({"subject": "CS"});
    let payload_b = json!({"subject": "MAT"});
    let payload_c = json!({"subject": "ENG"});

    // Insert A and B as Subject jobs
    helpers::insert_scrape_job(
        &pool,
        TargetType::Subject,
        payload_a.clone(),
        ScrapePriority::Medium,
        false,
        0,
        3,
    )
    .await;
    helpers::insert_scrape_job(
        &pool,
        TargetType::Subject,
        payload_b.clone(),
        ScrapePriority::Medium,
        false,
        0,
        3,
    )
    .await;
    // Insert C as a different target type
    helpers::insert_scrape_job(
        &pool,
        TargetType::SingleCrn,
        payload_c.clone(),
        ScrapePriority::Medium,
        false,
        0,
        3,
    )
    .await;

    let candidates = vec![payload_a.clone(), payload_b.clone(), payload_c.clone()];
    let existing = scrape_jobs::find_existing_job_payloads(TargetType::Subject, &candidates, &pool)
        .await
        .unwrap();

    assert!(existing.contains(&payload_a.to_string()));
    assert!(existing.contains(&payload_b.to_string()));
    // payload_c is SingleCrn, not Subject — should not match
    assert!(!existing.contains(&payload_c.to_string()));
}

#[sqlx::test]
async fn find_existing_payloads_includes_locked(pool: PgPool) {
    let payload = json!({"subject": "CS"});

    helpers::insert_scrape_job(
        &pool,
        TargetType::Subject,
        payload.clone(),
        ScrapePriority::Medium,
        true, // locked
        0,
        3,
    )
    .await;

    let candidates = vec![payload.clone()];
    let existing = scrape_jobs::find_existing_job_payloads(TargetType::Subject, &candidates, &pool)
        .await
        .unwrap();

    assert!(
        existing.contains(&payload.to_string()),
        "locked jobs should be included in deduplication"
    );
}

#[sqlx::test]
async fn find_existing_payloads_empty_candidates(pool: PgPool) {
    // Insert a job so the table isn't empty
    helpers::insert_scrape_job(
        &pool,
        TargetType::Subject,
        json!({"subject": "CS"}),
        ScrapePriority::Medium,
        false,
        0,
        3,
    )
    .await;

    let existing = scrape_jobs::find_existing_job_payloads(TargetType::Subject, &[], &pool)
        .await
        .unwrap();

    assert!(
        existing.is_empty(),
        "empty candidates should return empty result"
    );
}

// ── batch_insert_jobs ───────────────────────────────────────────────

#[sqlx::test]
async fn batch_insert_jobs_inserts_multiple(pool: PgPool) {
    let jobs = vec![
        (
            json!({"subject": "CS"}),
            TargetType::Subject,
            ScrapePriority::High,
        ),
        (
            json!({"subject": "MAT"}),
            TargetType::Subject,
            ScrapePriority::Medium,
        ),
        (
            json!({"crn": "12345"}),
            TargetType::SingleCrn,
            ScrapePriority::Low,
        ),
    ];

    scrape_jobs::batch_insert_jobs(&jobs, &pool).await.unwrap();

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM scrape_jobs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 3);
}

#[sqlx::test]
async fn batch_insert_jobs_empty_slice(pool: PgPool) {
    scrape_jobs::batch_insert_jobs(&[], &pool).await.unwrap();

    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM scrape_jobs")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 0);
}
