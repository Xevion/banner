//! `sqlx` models for the database schema.

use chrono::{DateTime, Utc};
use serde_json::Value;

#[allow(dead_code)]
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Course {
    pub id: i32,
    pub crn: String,
    pub subject: String,
    pub course_number: String,
    pub title: String,
    pub term_code: String,
    pub enrollment: i32,
    pub max_enrollment: i32,
    pub wait_count: i32,
    pub wait_capacity: i32,
    pub last_scraped_at: DateTime<Utc>,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct CourseMetric {
    pub id: i32,
    pub course_id: i32,
    pub timestamp: DateTime<Utc>,
    pub enrollment: i32,
    pub wait_count: i32,
    pub seats_available: i32,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct CourseAudit {
    pub id: i32,
    pub course_id: i32,
    pub timestamp: DateTime<Utc>,
    pub field_changed: String,
    pub old_value: String,
    pub new_value: String,
}

/// The priority level of a scrape job.
#[derive(sqlx::Type, Copy, Debug, Clone)]
#[sqlx(type_name = "scrape_priority", rename_all = "PascalCase")]
pub enum ScrapePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// The type of target for a scrape job, determining how the payload is interpreted.
#[derive(sqlx::Type, Copy, Debug, Clone)]
#[sqlx(type_name = "target_type", rename_all = "PascalCase")]
pub enum TargetType {
    Subject,
    CourseRange,
    CrnList,
    SingleCrn,
}

/// Represents a queryable job from the database.
#[allow(dead_code)]
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct ScrapeJob {
    pub id: i32,
    pub target_type: TargetType,
    pub target_payload: Value,
    pub priority: ScrapePriority,
    pub execute_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub locked_at: Option<DateTime<Utc>>,
}
