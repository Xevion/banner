//! `sqlx` models for the database schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;

/// Represents a meeting time stored as JSONB in the courses table.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DbMeetingTime {
    pub begin_time: Option<String>,
    pub end_time: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
    pub building: Option<String>,
    pub building_description: Option<String>,
    pub room: Option<String>,
    pub campus: Option<String>,
    pub meeting_type: String,
    pub meeting_schedule_type: String,
}

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
    // New scalar fields
    pub sequence_number: Option<String>,
    pub part_of_term: Option<String>,
    pub instructional_method: Option<String>,
    pub campus: Option<String>,
    pub credit_hours: Option<i32>,
    pub credit_hour_low: Option<i32>,
    pub credit_hour_high: Option<i32>,
    pub cross_list: Option<String>,
    pub cross_list_capacity: Option<i32>,
    pub cross_list_count: Option<i32>,
    pub link_identifier: Option<String>,
    pub is_section_linked: Option<bool>,
    // JSONB fields
    pub meeting_times: Value,
    pub attributes: Value,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Instructor {
    pub banner_id: String,
    pub display_name: String,
    pub email: Option<String>,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct CourseInstructor {
    pub course_id: i32,
    pub instructor_id: String,
    pub is_primary: bool,
}

/// Joined instructor data for a course (from course_instructors + instructors + rmp_professors).
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct CourseInstructorDetail {
    pub banner_id: String,
    pub display_name: String,
    pub email: Option<String>,
    pub is_primary: bool,
    pub avg_rating: Option<f32>,
    pub num_ratings: Option<i32>,
    /// Present when fetched via batch query; `None` for single-course queries.
    pub course_id: Option<i32>,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct ReferenceData {
    pub category: String,
    pub code: String,
    pub description: String,
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
    /// Number of retry attempts for this job (non-negative, enforced by CHECK constraint)
    pub retry_count: i32,
    /// Maximum number of retry attempts allowed (non-negative, enforced by CHECK constraint)
    pub max_retries: i32,
}
