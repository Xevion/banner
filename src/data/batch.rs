//! Batch database operations for improved performance.

use crate::banner::Course;
use crate::data::models::DbMeetingTime;
use crate::error::Result;
use sqlx::PgPool;
use std::collections::HashSet;
use std::time::Instant;
use tracing::info;

/// Convert a Banner API course's meeting times to the DB JSONB shape.
fn to_db_meeting_times(course: &Course) -> serde_json::Value {
    let meetings: Vec<DbMeetingTime> = course
        .meetings_faculty
        .iter()
        .map(|mf| {
            let mt = &mf.meeting_time;
            DbMeetingTime {
                begin_time: mt.begin_time.clone(),
                end_time: mt.end_time.clone(),
                start_date: mt.start_date.clone(),
                end_date: mt.end_date.clone(),
                monday: mt.monday,
                tuesday: mt.tuesday,
                wednesday: mt.wednesday,
                thursday: mt.thursday,
                friday: mt.friday,
                saturday: mt.saturday,
                sunday: mt.sunday,
                building: mt.building.clone(),
                building_description: mt.building_description.clone(),
                room: mt.room.clone(),
                campus: mt.campus.clone(),
                meeting_type: mt.meeting_type.clone(),
                meeting_schedule_type: mt.meeting_schedule_type.clone(),
            }
        })
        .collect();
    serde_json::to_value(meetings).unwrap_or_default()
}

/// Convert a Banner API course's section attributes to a JSONB array of code strings.
fn to_db_attributes(course: &Course) -> serde_json::Value {
    let codes: Vec<&str> = course
        .section_attributes
        .iter()
        .map(|a| a.code.as_str())
        .collect();
    serde_json::to_value(codes).unwrap_or_default()
}

/// Extract the campus code from the first meeting time (Banner doesn't put it on the course directly).
fn extract_campus_code(course: &Course) -> Option<String> {
    course
        .meetings_faculty
        .first()
        .and_then(|mf| mf.meeting_time.campus.clone())
}

/// Batch upsert courses in a single database query.
///
/// Performs a bulk INSERT...ON CONFLICT DO UPDATE for all courses, including
/// new fields (meeting times, attributes, instructor data). Returns the
/// database IDs for all upserted courses (in input order) so instructors
/// can be linked.
///
/// # Performance
/// - Reduces N database round-trips to 3 (courses, instructors, junction)
/// - Typical usage: 50-200 courses per batch
pub async fn batch_upsert_courses(courses: &[Course], db_pool: &PgPool) -> Result<()> {
    if courses.is_empty() {
        info!("No courses to upsert, skipping batch operation");
        return Ok(());
    }

    let start = Instant::now();
    let course_count = courses.len();

    // Step 1: Upsert courses with all fields, returning IDs
    let course_ids = upsert_courses(courses, db_pool).await?;

    // Step 2: Upsert instructors (deduplicated across batch)
    upsert_instructors(courses, db_pool).await?;

    // Step 3: Link courses to instructors via junction table
    upsert_course_instructors(courses, &course_ids, db_pool).await?;

    let duration = start.elapsed();
    info!(
        courses_count = course_count,
        duration_ms = duration.as_millis(),
        "Batch upserted courses with instructors"
    );

    Ok(())
}

/// Upsert all courses and return their database IDs in input order.
async fn upsert_courses(courses: &[Course], db_pool: &PgPool) -> Result<Vec<i32>> {
    let crns: Vec<&str> = courses
        .iter()
        .map(|c| c.course_reference_number.as_str())
        .collect();
    let subjects: Vec<&str> = courses.iter().map(|c| c.subject.as_str()).collect();
    let course_numbers: Vec<&str> = courses.iter().map(|c| c.course_number.as_str()).collect();
    let titles: Vec<&str> = courses.iter().map(|c| c.course_title.as_str()).collect();
    let term_codes: Vec<&str> = courses.iter().map(|c| c.term.as_str()).collect();
    let enrollments: Vec<i32> = courses.iter().map(|c| c.enrollment).collect();
    let max_enrollments: Vec<i32> = courses.iter().map(|c| c.maximum_enrollment).collect();
    let wait_counts: Vec<i32> = courses.iter().map(|c| c.wait_count).collect();
    let wait_capacities: Vec<i32> = courses.iter().map(|c| c.wait_capacity).collect();

    // New scalar fields
    let sequence_numbers: Vec<Option<&str>> = courses
        .iter()
        .map(|c| Some(c.sequence_number.as_str()))
        .collect();
    let parts_of_term: Vec<Option<&str>> = courses
        .iter()
        .map(|c| Some(c.part_of_term.as_str()))
        .collect();
    let instructional_methods: Vec<Option<&str>> = courses
        .iter()
        .map(|c| Some(c.instructional_method.as_str()))
        .collect();
    let campuses: Vec<Option<String>> = courses.iter().map(extract_campus_code).collect();
    let credit_hours: Vec<Option<i32>> = courses.iter().map(|c| c.credit_hours).collect();
    let credit_hour_lows: Vec<Option<i32>> = courses.iter().map(|c| c.credit_hour_low).collect();
    let credit_hour_highs: Vec<Option<i32>> = courses.iter().map(|c| c.credit_hour_high).collect();
    let cross_lists: Vec<Option<&str>> = courses.iter().map(|c| c.cross_list.as_deref()).collect();
    let cross_list_capacities: Vec<Option<i32>> =
        courses.iter().map(|c| c.cross_list_capacity).collect();
    let cross_list_counts: Vec<Option<i32>> = courses.iter().map(|c| c.cross_list_count).collect();
    let link_identifiers: Vec<Option<&str>> = courses
        .iter()
        .map(|c| c.link_identifier.as_deref())
        .collect();
    let is_section_linkeds: Vec<Option<bool>> =
        courses.iter().map(|c| Some(c.is_section_linked)).collect();

    // JSONB fields
    let meeting_times_json: Vec<serde_json::Value> =
        courses.iter().map(to_db_meeting_times).collect();
    let attributes_json: Vec<serde_json::Value> = courses.iter().map(to_db_attributes).collect();

    let rows = sqlx::query_scalar::<_, i32>(
        r#"
        INSERT INTO courses (
            crn, subject, course_number, title, term_code,
            enrollment, max_enrollment, wait_count, wait_capacity, last_scraped_at,
            sequence_number, part_of_term, instructional_method, campus,
            credit_hours, credit_hour_low, credit_hour_high,
            cross_list, cross_list_capacity, cross_list_count,
            link_identifier, is_section_linked,
            meeting_times, attributes
        )
        SELECT
            v.crn, v.subject, v.course_number, v.title, v.term_code,
            v.enrollment, v.max_enrollment, v.wait_count, v.wait_capacity, NOW(),
            v.sequence_number, v.part_of_term, v.instructional_method, v.campus,
            v.credit_hours, v.credit_hour_low, v.credit_hour_high,
            v.cross_list, v.cross_list_capacity, v.cross_list_count,
            v.link_identifier, v.is_section_linked,
            v.meeting_times, v.attributes
        FROM UNNEST(
            $1::text[], $2::text[], $3::text[], $4::text[], $5::text[],
            $6::int4[], $7::int4[], $8::int4[], $9::int4[],
            $10::text[], $11::text[], $12::text[], $13::text[],
            $14::int4[], $15::int4[], $16::int4[],
            $17::text[], $18::int4[], $19::int4[],
            $20::text[], $21::bool[],
            $22::jsonb[], $23::jsonb[]
        ) AS v(
            crn, subject, course_number, title, term_code,
            enrollment, max_enrollment, wait_count, wait_capacity,
            sequence_number, part_of_term, instructional_method, campus,
            credit_hours, credit_hour_low, credit_hour_high,
            cross_list, cross_list_capacity, cross_list_count,
            link_identifier, is_section_linked,
            meeting_times, attributes
        )
        ON CONFLICT (crn, term_code)
        DO UPDATE SET
            subject = EXCLUDED.subject,
            course_number = EXCLUDED.course_number,
            title = EXCLUDED.title,
            enrollment = EXCLUDED.enrollment,
            max_enrollment = EXCLUDED.max_enrollment,
            wait_count = EXCLUDED.wait_count,
            wait_capacity = EXCLUDED.wait_capacity,
            last_scraped_at = EXCLUDED.last_scraped_at,
            sequence_number = EXCLUDED.sequence_number,
            part_of_term = EXCLUDED.part_of_term,
            instructional_method = EXCLUDED.instructional_method,
            campus = EXCLUDED.campus,
            credit_hours = EXCLUDED.credit_hours,
            credit_hour_low = EXCLUDED.credit_hour_low,
            credit_hour_high = EXCLUDED.credit_hour_high,
            cross_list = EXCLUDED.cross_list,
            cross_list_capacity = EXCLUDED.cross_list_capacity,
            cross_list_count = EXCLUDED.cross_list_count,
            link_identifier = EXCLUDED.link_identifier,
            is_section_linked = EXCLUDED.is_section_linked,
            meeting_times = EXCLUDED.meeting_times,
            attributes = EXCLUDED.attributes
        RETURNING id
        "#,
    )
    .bind(&crns)
    .bind(&subjects)
    .bind(&course_numbers)
    .bind(&titles)
    .bind(&term_codes)
    .bind(&enrollments)
    .bind(&max_enrollments)
    .bind(&wait_counts)
    .bind(&wait_capacities)
    .bind(&sequence_numbers)
    .bind(&parts_of_term)
    .bind(&instructional_methods)
    .bind(&campuses)
    .bind(&credit_hours)
    .bind(&credit_hour_lows)
    .bind(&credit_hour_highs)
    .bind(&cross_lists)
    .bind(&cross_list_capacities)
    .bind(&cross_list_counts)
    .bind(&link_identifiers)
    .bind(&is_section_linkeds)
    .bind(&meeting_times_json)
    .bind(&attributes_json)
    .fetch_all(db_pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to batch upsert courses: {}", e))?;

    Ok(rows)
}

/// Deduplicate and upsert all instructors from the batch.
async fn upsert_instructors(courses: &[Course], db_pool: &PgPool) -> Result<()> {
    let mut seen = HashSet::new();
    let mut banner_ids = Vec::new();
    let mut display_names = Vec::new();
    let mut emails: Vec<Option<&str>> = Vec::new();

    for course in courses {
        for faculty in &course.faculty {
            if seen.insert(faculty.banner_id.as_str()) {
                banner_ids.push(faculty.banner_id.as_str());
                display_names.push(faculty.display_name.as_str());
                emails.push(faculty.email_address.as_deref());
            }
        }
    }

    if banner_ids.is_empty() {
        return Ok(());
    }

    sqlx::query(
        r#"
        INSERT INTO instructors (banner_id, display_name, email)
        SELECT * FROM UNNEST($1::text[], $2::text[], $3::text[])
        ON CONFLICT (banner_id)
        DO UPDATE SET
            display_name = EXCLUDED.display_name,
            email = COALESCE(EXCLUDED.email, instructors.email)
        "#,
    )
    .bind(&banner_ids)
    .bind(&display_names)
    .bind(&emails)
    .execute(db_pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to batch upsert instructors: {}", e))?;

    Ok(())
}

/// Link courses to their instructors via the junction table.
async fn upsert_course_instructors(
    courses: &[Course],
    course_ids: &[i32],
    db_pool: &PgPool,
) -> Result<()> {
    let mut cids = Vec::new();
    let mut iids = Vec::new();
    let mut primaries = Vec::new();

    for (course, &course_id) in courses.iter().zip(course_ids) {
        for faculty in &course.faculty {
            cids.push(course_id);
            iids.push(faculty.banner_id.as_str());
            primaries.push(faculty.primary_indicator);
        }
    }

    if cids.is_empty() {
        return Ok(());
    }

    // Delete existing links for these courses then re-insert.
    // This handles instructor changes cleanly.
    sqlx::query("DELETE FROM course_instructors WHERE course_id = ANY($1)")
        .bind(&cids)
        .execute(db_pool)
        .await?;

    sqlx::query(
        r#"
        INSERT INTO course_instructors (course_id, instructor_id, is_primary)
        SELECT * FROM UNNEST($1::int4[], $2::text[], $3::bool[])
        ON CONFLICT (course_id, instructor_id)
        DO UPDATE SET is_primary = EXCLUDED.is_primary
        "#,
    )
    .bind(&cids)
    .bind(&iids)
    .bind(&primaries)
    .execute(db_pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to batch upsert course_instructors: {}", e))?;

    Ok(())
}
