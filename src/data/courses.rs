//! Database query functions for courses, used by the web API.

use crate::data::models::Course;
use crate::error::Result;
use sqlx::PgPool;

/// Search courses by term with optional filters.
///
/// Returns `(courses, total_count)` for pagination. Uses FTS tsvector for word
/// search and falls back to trigram ILIKE for substring matching.
#[allow(clippy::too_many_arguments)]
pub async fn search_courses(
    db_pool: &PgPool,
    term_code: &str,
    subject: Option<&str>,
    title_query: Option<&str>,
    course_number_low: Option<i32>,
    course_number_high: Option<i32>,
    open_only: bool,
    instructional_method: Option<&str>,
    campus: Option<&str>,
    limit: i32,
    offset: i32,
    order_by: &str,
) -> Result<(Vec<Course>, i64)> {
    // Build WHERE clauses dynamically via parameter binding + COALESCE trick:
    // each optional filter uses ($N IS NULL OR column = $N) so NULL means "no filter".
    //
    // ORDER BY is interpolated as a string since column names can't be bound as
    // parameters. The caller must provide a safe, pre-validated clause (see
    // `sort_clause` in routes.rs).
    let query = format!(
        r#"
        SELECT *
        FROM courses
        WHERE term_code = $1
          AND ($2::text IS NULL OR subject = $2)
          AND ($3::text IS NULL OR title_search @@ plainto_tsquery('simple', $3) OR title ILIKE '%' || $3 || '%')
          AND ($4::int IS NULL OR course_number::int >= $4)
          AND ($5::int IS NULL OR course_number::int <= $5)
          AND ($6::bool = false OR max_enrollment > enrollment)
          AND ($7::text IS NULL OR instructional_method = $7)
          AND ($8::text IS NULL OR campus = $8)
        ORDER BY {order_by}
        LIMIT $9 OFFSET $10
        "#
    );

    let courses = sqlx::query_as::<_, Course>(&query)
        .bind(term_code)
        .bind(subject)
        .bind(title_query)
        .bind(course_number_low)
        .bind(course_number_high)
        .bind(open_only)
        .bind(instructional_method)
        .bind(campus)
        .bind(limit)
        .bind(offset)
        .fetch_all(db_pool)
        .await?;

    let total: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM courses
        WHERE term_code = $1
          AND ($2::text IS NULL OR subject = $2)
          AND ($3::text IS NULL OR title_search @@ plainto_tsquery('simple', $3) OR title ILIKE '%' || $3 || '%')
          AND ($4::int IS NULL OR course_number::int >= $4)
          AND ($5::int IS NULL OR course_number::int <= $5)
          AND ($6::bool = false OR max_enrollment > enrollment)
          AND ($7::text IS NULL OR instructional_method = $7)
          AND ($8::text IS NULL OR campus = $8)
        "#,
    )
    .bind(term_code)
    .bind(subject)
    .bind(title_query)
    .bind(course_number_low)
    .bind(course_number_high)
    .bind(open_only)
    .bind(instructional_method)
    .bind(campus)
    .fetch_one(db_pool)
    .await?;

    Ok((courses, total.0))
}

/// Get a single course by CRN and term.
pub async fn get_course_by_crn(
    db_pool: &PgPool,
    crn: &str,
    term_code: &str,
) -> Result<Option<Course>> {
    let course =
        sqlx::query_as::<_, Course>("SELECT * FROM courses WHERE crn = $1 AND term_code = $2")
            .bind(crn)
            .bind(term_code)
            .fetch_optional(db_pool)
            .await?;
    Ok(course)
}

/// Get instructors for a course by course ID.
///
/// Returns `(banner_id, display_name, email, is_primary, rmp_avg_rating, rmp_num_ratings)` tuples.
pub async fn get_course_instructors(
    db_pool: &PgPool,
    course_id: i32,
) -> Result<
    Vec<(
        String,
        String,
        Option<String>,
        bool,
        Option<f32>,
        Option<i32>,
    )>,
> {
    let rows: Vec<(
        String,
        String,
        Option<String>,
        bool,
        Option<f32>,
        Option<i32>,
    )> = sqlx::query_as(
        r#"
        SELECT i.banner_id, i.display_name, i.email, ci.is_primary,
               rp.avg_rating, rp.num_ratings
        FROM course_instructors ci
        JOIN instructors i ON i.banner_id = ci.instructor_id
        LEFT JOIN rmp_professors rp ON rp.legacy_id = i.rmp_legacy_id
        WHERE ci.course_id = $1
        ORDER BY ci.is_primary DESC, i.display_name
        "#,
    )
    .bind(course_id)
    .fetch_all(db_pool)
    .await?;
    Ok(rows)
}

/// Get all distinct term codes that have courses in the DB.
pub async fn get_available_terms(db_pool: &PgPool) -> Result<Vec<String>> {
    let rows: Vec<(String,)> =
        sqlx::query_as("SELECT DISTINCT term_code FROM courses ORDER BY term_code DESC")
            .fetch_all(db_pool)
            .await?;
    Ok(rows.into_iter().map(|(tc,)| tc).collect())
}
