//! `sqlx` models for the database schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use ts_rs::TS;

/// Serialize an `i64` as a string to avoid JavaScript precision loss for values exceeding 2^53.
fn serialize_i64_as_string<S: Serializer>(value: &i64, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&value.to_string())
}

/// Deserialize an `i64` from either a number or a string.
fn deserialize_i64_from_string<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<i64, D::Error> {
    use serde::de;

    struct I64OrStringVisitor;

    impl<'de> de::Visitor<'de> for I64OrStringVisitor {
        type Value = i64;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an integer or a string containing an integer")
        }

        fn visit_i64<E: de::Error>(self, value: i64) -> Result<i64, E> {
            Ok(value)
        }

        fn visit_u64<E: de::Error>(self, value: u64) -> Result<i64, E> {
            i64::try_from(value).map_err(|_| E::custom(format!("u64 {value} out of i64 range")))
        }

        fn visit_str<E: de::Error>(self, value: &str) -> Result<i64, E> {
            value.parse().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_any(I64OrStringVisitor)
}

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
    pub id: i32,
    pub display_name: String,
    pub email: String,
    pub rmp_match_status: String,
}

#[allow(dead_code)]
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct CourseInstructor {
    pub course_id: i32,
    pub instructor_id: i32,
    pub banner_id: String,
    pub is_primary: bool,
}

/// Joined instructor data for a course (from course_instructors + instructors + rmp_professors).
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct CourseInstructorDetail {
    pub instructor_id: i32,
    pub banner_id: String,
    pub display_name: String,
    pub email: String,
    pub is_primary: bool,
    pub avg_rating: Option<f32>,
    pub num_ratings: Option<i32>,
    pub rmp_legacy_id: Option<i32>,
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

/// Aggregate counts returned by batch upsert, used for scrape job result logging.
#[derive(Debug, Clone, Default)]
pub struct UpsertCounts {
    pub courses_fetched: i32,
    pub courses_changed: i32,
    pub courses_unchanged: i32,
    pub audits_generated: i32,
    pub metrics_generated: i32,
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

/// Computed status for a scrape job, derived from existing fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ScrapeJobStatus {
    Processing,
    StaleLock,
    Exhausted,
    Scheduled,
    Pending,
}

/// How long a lock can be held before it is considered stale (mirrors `scrape_jobs::LOCK_EXPIRY`).
const LOCK_EXPIRY_SECS: i64 = 10 * 60;

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
    /// When the job last entered the "ready to pick up" state.
    /// Set to NOW() on creation; updated to NOW() on retry.
    pub queued_at: DateTime<Utc>,
}

impl ScrapeJob {
    /// Compute the current status of this job from its fields.
    pub fn status(&self) -> ScrapeJobStatus {
        let now = Utc::now();
        match self.locked_at {
            Some(locked) if (now - locked).num_seconds() < LOCK_EXPIRY_SECS => {
                ScrapeJobStatus::Processing
            }
            Some(_) => ScrapeJobStatus::StaleLock,
            None if self.retry_count >= self.max_retries && self.max_retries > 0 => {
                ScrapeJobStatus::Exhausted
            }
            None if self.execute_at > now => ScrapeJobStatus::Scheduled,
            None => ScrapeJobStatus::Pending,
        }
    }
}

/// A user authenticated via Discord OAuth.
#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct User {
    #[serde(
        serialize_with = "serialize_i64_as_string",
        deserialize_with = "deserialize_i64_from_string"
    )]
    #[ts(type = "string")]
    pub discord_id: i64,
    pub discord_username: String,
    pub discord_avatar_hash: Option<String>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A server-side session for an authenticated user.
#[allow(dead_code)] // Fields read via sqlx::FromRow; some only used in DB queries
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct UserSession {
    pub id: String,
    pub user_id: i64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}
