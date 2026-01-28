use banner::banner::Course;
use banner::data::models::{ScrapePriority, TargetType};
use chrono::Utc;
use sqlx::PgPool;

/// Build a test `Course` (Banner API model) with sensible defaults.
///
/// Only the fields used by `batch_upsert_courses` need meaningful values;
/// the rest are filled with harmless placeholders.
pub fn make_course(
    crn: &str,
    term: &str,
    subject: &str,
    course_number: &str,
    title: &str,
    enrollment: i32,
    max_enrollment: i32,
    wait_count: i32,
    wait_capacity: i32,
) -> Course {
    Course {
        id: 0,
        term: term.to_owned(),
        term_desc: String::new(),
        course_reference_number: crn.to_owned(),
        part_of_term: "1".to_owned(),
        course_number: course_number.to_owned(),
        subject: subject.to_owned(),
        subject_description: subject.to_owned(),
        sequence_number: "001".to_owned(),
        campus_description: "Main Campus".to_owned(),
        schedule_type_description: "Lecture".to_owned(),
        course_title: title.to_owned(),
        credit_hours: Some(3),
        maximum_enrollment: max_enrollment,
        enrollment,
        seats_available: max_enrollment - enrollment,
        wait_capacity,
        wait_count,
        cross_list: None,
        cross_list_capacity: None,
        cross_list_count: None,
        cross_list_available: None,
        credit_hour_high: None,
        credit_hour_low: None,
        credit_hour_indicator: None,
        open_section: enrollment < max_enrollment,
        link_identifier: None,
        is_section_linked: false,
        subject_course: format!("{subject}{course_number}"),
        reserved_seat_summary: None,
        instructional_method: "FF".to_owned(),
        instructional_method_description: "Face to Face".to_owned(),
        section_attributes: vec![],
        faculty: vec![],
        meetings_faculty: vec![],
    }
}

/// Insert a scrape job row directly via SQL, returning the generated ID.
pub async fn insert_scrape_job(
    pool: &PgPool,
    target_type: TargetType,
    payload: serde_json::Value,
    priority: ScrapePriority,
    locked: bool,
    retry_count: i32,
    max_retries: i32,
) -> i32 {
    let locked_at = if locked { Some(Utc::now()) } else { None };

    let (id,): (i32,) = sqlx::query_as(
        "INSERT INTO scrape_jobs (target_type, target_payload, priority, execute_at, locked_at, retry_count, max_retries)
         VALUES ($1, $2, $3, NOW(), $4, $5, $6)
         RETURNING id",
    )
    .bind(target_type)
    .bind(payload)
    .bind(priority)
    .bind(locked_at)
    .bind(retry_count)
    .bind(max_retries)
    .fetch_one(pool)
    .await
    .expect("insert_scrape_job failed");

    id
}
