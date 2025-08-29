//! Diesel models for the database schema.

use crate::data::schema::{course_audits, course_metrics, courses, scrape_jobs};
use chrono::{DateTime, Utc};
use diesel::{Insertable, Queryable, QueryableByName, Selectable};
use serde_json::Value;

#[derive(Queryable, Selectable)]
#[diesel(table_name = courses)]
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

#[derive(Insertable)]
#[diesel(table_name = courses)]
pub struct NewCourse<'a> {
    pub crn: &'a str,
    pub subject: &'a str,
    pub course_number: &'a str,
    pub title: &'a str,
    pub term_code: &'a str,
    pub enrollment: i32,
    pub max_enrollment: i32,
    pub wait_count: i32,
    pub wait_capacity: i32,
    pub last_scraped_at: DateTime<Utc>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = course_metrics)]
#[diesel(belongs_to(Course))]
pub struct CourseMetric {
    pub id: i32,
    pub course_id: i32,
    pub timestamp: DateTime<Utc>,
    pub enrollment: i32,
    pub wait_count: i32,
    pub seats_available: i32,
}

#[derive(Insertable)]
#[diesel(table_name = course_metrics)]
pub struct NewCourseMetric {
    pub course_id: i32,
    pub timestamp: DateTime<Utc>,
    pub enrollment: i32,
    pub wait_count: i32,
    pub seats_available: i32,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = course_audits)]
#[diesel(belongs_to(Course))]
pub struct CourseAudit {
    pub id: i32,
    pub course_id: i32,
    pub timestamp: DateTime<Utc>,
    pub field_changed: String,
    pub old_value: String,
    pub new_value: String,
}

#[derive(Insertable)]
#[diesel(table_name = course_audits)]
pub struct NewCourseAudit<'a> {
    pub course_id: i32,
    pub timestamp: DateTime<Utc>,
    pub field_changed: &'a str,
    pub old_value: &'a str,
    pub new_value: &'a str,
}

/// The priority level of a scrape job.
#[derive(diesel_derive_enum::DbEnum, Copy, Debug, Clone)]
pub enum ScrapePriority {
    Low,
    Medium,
    High,
    Critical,
}

/// The type of target for a scrape job, determining how the payload is interpreted.
#[derive(diesel_derive_enum::DbEnum, Copy, Debug, Clone)]
pub enum TargetType {
    Subject,
    CourseRange,
    CrnList,
    SingleCrn,
}

/// Represents a queryable job from the database.
#[derive(Debug, Clone, Queryable, QueryableByName)]
#[diesel(table_name = scrape_jobs)]
pub struct ScrapeJob {
    pub id: i32,
    pub target_type: TargetType,
    pub target_payload: Value,
    pub priority: ScrapePriority,
    pub execute_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub locked_at: Option<DateTime<Utc>>,
}

/// Represents a new job to be inserted into the database.
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = scrape_jobs)]
pub struct NewScrapeJob {
    pub target_type: TargetType,
    #[diesel(sql_type = diesel::sql_types::Jsonb)]
    pub target_payload: Value,
    pub priority: ScrapePriority,
    pub execute_at: DateTime<Utc>,
}
