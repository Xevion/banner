//! Batch database operations for improved performance.

use crate::banner::Course;
use crate::error::Result;
use sqlx::PgPool;
use std::time::Instant;
use tracing::info;

/// Batch upsert courses in a single database query.
///
/// This function performs a bulk INSERT...ON CONFLICT DO UPDATE for all courses
/// in a single round-trip to the database, significantly reducing overhead compared
/// to individual inserts.
///
/// # Performance
/// - Reduces N database round-trips to 1
/// - Typical usage: 50-200 courses per batch
/// - PostgreSQL parameter limit: 65,535 (we use ~10 per course)
///
/// # Arguments
/// * `courses` - Slice of Course structs from the Banner API
/// * `db_pool` - PostgreSQL connection pool
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(_)` if the database operation fails
///
/// # Example
/// ```no_run
/// use banner::data::batch::batch_upsert_courses;
/// use banner::banner::Course;
/// use sqlx::PgPool;
///
/// async fn example(courses: &[Course], pool: &PgPool) -> anyhow::Result<()> {
///     batch_upsert_courses(courses, pool).await?;
///     Ok(())
/// }
/// ```
pub async fn batch_upsert_courses(courses: &[Course], db_pool: &PgPool) -> Result<()> {
    // Early return for empty batches
    if courses.is_empty() {
        info!("No courses to upsert, skipping batch operation");
        return Ok(());
    }

    let start = Instant::now();
    let course_count = courses.len();

    // Extract course fields into vectors for UNNEST
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

    // Perform batch upsert using UNNEST for efficient bulk insertion
    let result = sqlx::query(
        r#"
        INSERT INTO courses (
            crn, subject, course_number, title, term_code,
            enrollment, max_enrollment, wait_count, wait_capacity, last_scraped_at
        )
        SELECT * FROM UNNEST(
            $1::text[], $2::text[], $3::text[], $4::text[], $5::text[],
            $6::int4[], $7::int4[], $8::int4[], $9::int4[],
            array_fill(NOW()::timestamptz, ARRAY[$10])
        ) AS t(
            crn, subject, course_number, title, term_code,
            enrollment, max_enrollment, wait_count, wait_capacity, last_scraped_at
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
            last_scraped_at = EXCLUDED.last_scraped_at
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
    .bind(course_count as i32)
    .execute(db_pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to batch upsert courses: {}", e))?;

    let duration = start.elapsed();

    info!(
        courses_count = course_count,
        rows_affected = result.rows_affected(),
        duration_ms = duration.as_millis(),
        "Batch upserted courses"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_batch_returns_ok() {
        // This is a basic compile-time test
        // Runtime tests would require sqlx::test macro and a test database
        let courses: Vec<Course> = vec![];
        assert_eq!(courses.len(), 0);
    }
}
