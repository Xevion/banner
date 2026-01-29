//! Database query functions for courses, used by the web API.

use crate::data::models::{Course, CourseInstructorDetail};
use crate::error::Result;
use sqlx::PgPool;
use std::collections::HashMap;

/// Column to sort search results by.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortColumn {
    CourseCode,
    Title,
    Instructor,
    Time,
    Seats,
}

/// Sort direction.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Shared WHERE clause for course search filters.
///
/// Parameters $1-$8 match the bind order in `search_courses`.
const SEARCH_WHERE: &str = r#"
    WHERE term_code = $1
      AND ($2::text[] IS NULL OR subject = ANY($2))
      AND ($3::text IS NULL OR title_search @@ plainto_tsquery('simple', $3) OR title ILIKE '%' || $3 || '%')
      AND ($4::int IS NULL OR course_number::int >= $4)
      AND ($5::int IS NULL OR course_number::int <= $5)
      AND ($6::bool = false OR max_enrollment > enrollment)
      AND ($7::text IS NULL OR instructional_method = $7)
      AND ($8::text IS NULL OR campus = $8)
"#;

/// Build a safe ORDER BY clause from typed sort parameters.
///
/// All column names are hardcoded string literals — no caller input is interpolated.
fn sort_clause(column: Option<SortColumn>, direction: Option<SortDirection>) -> String {
    let dir = match direction.unwrap_or(SortDirection::Asc) {
        SortDirection::Asc => "ASC",
        SortDirection::Desc => "DESC",
    };

    match column {
        Some(SortColumn::CourseCode) => {
            format!("subject {dir}, course_number {dir}, sequence_number {dir}")
        }
        Some(SortColumn::Title) => format!("title {dir}"),
        Some(SortColumn::Instructor) => {
            format!(
                "(SELECT i.display_name FROM course_instructors ci \
                 JOIN instructors i ON i.banner_id = ci.instructor_id \
                 WHERE ci.course_id = courses.id AND ci.is_primary = true \
                 LIMIT 1) {dir} NULLS LAST"
            )
        }
        Some(SortColumn::Time) => {
            format!("(meeting_times->0->>'begin_time') {dir} NULLS LAST")
        }
        Some(SortColumn::Seats) => {
            format!("(max_enrollment - enrollment) {dir}")
        }
        None => "subject ASC, course_number ASC, sequence_number ASC".to_string(),
    }
}

/// Search courses by term with optional filters.
///
/// Returns `(courses, total_count)` for pagination. Uses FTS tsvector for word
/// search and falls back to trigram ILIKE for substring matching.
#[allow(clippy::too_many_arguments)]
pub async fn search_courses(
    db_pool: &PgPool,
    term_code: &str,
    subject: Option<&[String]>,
    title_query: Option<&str>,
    course_number_low: Option<i32>,
    course_number_high: Option<i32>,
    open_only: bool,
    instructional_method: Option<&str>,
    campus: Option<&str>,
    limit: i32,
    offset: i32,
    sort_by: Option<SortColumn>,
    sort_dir: Option<SortDirection>,
) -> Result<(Vec<Course>, i64)> {
    let order_by = sort_clause(sort_by, sort_dir);

    let data_query = format!(
        "SELECT * FROM courses {SEARCH_WHERE} ORDER BY {order_by} LIMIT $9 OFFSET $10"
    );
    let count_query = format!("SELECT COUNT(*) FROM courses {SEARCH_WHERE}");

    let courses = sqlx::query_as::<_, Course>(&data_query)
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

    let total: (i64,) = sqlx::query_as(&count_query)
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

/// Get instructors for a single course by course ID.
pub async fn get_course_instructors(
    db_pool: &PgPool,
    course_id: i32,
) -> Result<Vec<CourseInstructorDetail>> {
    let rows = sqlx::query_as::<_, CourseInstructorDetail>(
        r#"
        SELECT i.banner_id, i.display_name, i.email, ci.is_primary,
               rp.avg_rating, rp.num_ratings,
               ci.course_id
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

/// Batch-fetch instructors for multiple courses in a single query.
///
/// Returns a map of `course_id → Vec<CourseInstructorDetail>`.
pub async fn get_instructors_for_courses(
    db_pool: &PgPool,
    course_ids: &[i32],
) -> Result<HashMap<i32, Vec<CourseInstructorDetail>>> {
    if course_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let rows = sqlx::query_as::<_, CourseInstructorDetail>(
        r#"
        SELECT i.banner_id, i.display_name, i.email, ci.is_primary,
               rp.avg_rating, rp.num_ratings,
               ci.course_id
        FROM course_instructors ci
        JOIN instructors i ON i.banner_id = ci.instructor_id
        LEFT JOIN rmp_professors rp ON rp.legacy_id = i.rmp_legacy_id
        WHERE ci.course_id = ANY($1)
        ORDER BY ci.course_id, ci.is_primary DESC, i.display_name
        "#,
    )
    .bind(course_ids)
    .fetch_all(db_pool)
    .await?;

    let mut map: HashMap<i32, Vec<CourseInstructorDetail>> = HashMap::new();
    for row in rows {
        // course_id is always present in the batch query
        let cid = row.course_id.unwrap_or_default();
        map.entry(cid).or_default().push(row);
    }
    Ok(map)
}

/// Get subjects for a term, sorted by total enrollment (descending).
///
/// Returns only subjects that have courses in the given term, with their
/// descriptions from reference_data and enrollment totals for ranking.
pub async fn get_subjects_by_enrollment(
    db_pool: &PgPool,
    term_code: &str,
) -> Result<Vec<(String, String, i64)>> {
    let rows: Vec<(String, String, i64)> = sqlx::query_as(
        r#"
        SELECT c.subject,
               COALESCE(rd.description, c.subject),
               COALESCE(SUM(c.enrollment), 0) as total_enrollment
        FROM courses c
        LEFT JOIN reference_data rd ON rd.category = 'subject' AND rd.code = c.subject
        WHERE c.term_code = $1
        GROUP BY c.subject, rd.description
        ORDER BY total_enrollment DESC
        "#,
    )
    .bind(term_code)
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
