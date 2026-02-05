//! Audit log stream logic.

use sqlx::PgPool;

use crate::web::audit::AuditLogEntry;
use crate::web::stream::filters::AuditLogFilter;

const DEFAULT_AUDIT_LIMIT: i32 = 200;
const MAX_AUDIT_LIMIT: i32 = 500;

#[derive(sqlx::FromRow)]
struct AuditRow {
    id: i32,
    course_id: i32,
    timestamp: chrono::DateTime<chrono::Utc>,
    field_changed: String,
    old_value: String,
    new_value: String,
    subject: Option<String>,
    course_number: Option<String>,
    crn: Option<String>,
    title: Option<String>,
    term_code: Option<String>,
}

pub async fn build_snapshot(
    db_pool: &PgPool,
    filter: &AuditLogFilter,
) -> Result<Vec<AuditLogEntry>, sqlx::Error> {
    let limit = filter
        .limit
        .unwrap_or(DEFAULT_AUDIT_LIMIT)
        .clamp(1, MAX_AUDIT_LIMIT);

    let rows: Vec<AuditRow> = sqlx::query_as(
        "SELECT a.id, a.course_id, a.timestamp, a.field_changed, a.old_value, a.new_value, \
                c.subject, c.course_number, c.crn, c.title, c.term_code \
         FROM course_audits a \
         LEFT JOIN courses c ON c.id = a.course_id \
         WHERE ($1::timestamptz IS NULL OR a.timestamp > $1) \
         ORDER BY a.timestamp DESC LIMIT $2",
    )
    .bind(filter.since_dt)
    .bind(limit)
    .fetch_all(db_pool)
    .await?;

    let entries = rows
        .into_iter()
        .filter(|row| {
            filter_matches(
                filter,
                &row.field_changed,
                row.subject.as_deref(),
                row.term_code.as_deref(),
            )
        })
        .map(|row| AuditLogEntry {
            id: row.id,
            course_id: row.course_id,
            timestamp: row.timestamp.to_rfc3339(),
            field_changed: row.field_changed,
            old_value: row.old_value,
            new_value: row.new_value,
            subject: row.subject,
            course_number: row.course_number,
            crn: row.crn,
            course_title: row.title,
            term_code: row.term_code,
        })
        .collect();

    Ok(entries)
}

pub fn filter_entries(filter: &AuditLogFilter, entries: &[AuditLogEntry]) -> Vec<AuditLogEntry> {
    entries
        .iter()
        .filter(|entry| entry_matches(filter, entry))
        .cloned()
        .collect()
}

pub fn entry_matches(filter: &AuditLogFilter, entry: &AuditLogEntry) -> bool {
    if let Some(ref since) = filter.since_dt
        && let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(&entry.timestamp)
        && timestamp.with_timezone(&chrono::Utc) <= *since
    {
        return false;
    }

    filter_matches(
        filter,
        &entry.field_changed,
        entry.subject.as_deref(),
        entry.term_code.as_deref(),
    )
}

fn filter_matches(
    filter: &AuditLogFilter,
    field_changed: &str,
    subject: Option<&str>,
    term_code: Option<&str>,
) -> bool {
    if let Some(ref fields) = filter.field_changed
        && !fields.is_empty()
        && !fields.iter().any(|f| f == field_changed)
    {
        return false;
    }

    if let Some(ref subjects) = filter.subject
        && !subjects.is_empty()
    {
        let Some(subject) = subject else {
            return false;
        };
        if !subjects.iter().any(|f| f == subject) {
            return false;
        }
    }

    if let Some(ref term) = filter.term
        && term_code != Some(term.as_str())
    {
        return false;
    }

    true
}
