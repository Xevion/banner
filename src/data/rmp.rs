//! Database operations for RateMyProfessors data.

use crate::error::Result;
use crate::rmp::RmpProfessor;
use sqlx::PgPool;
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};

/// Bulk upsert RMP professors using the UNNEST pattern.
///
/// Deduplicates by `legacy_id` before inserting — the RMP API can return
/// the same professor on multiple pages.
pub async fn batch_upsert_rmp_professors(
    professors: &[RmpProfessor],
    db_pool: &PgPool,
) -> Result<()> {
    if professors.is_empty() {
        return Ok(());
    }

    // Deduplicate: keep last occurrence per legacy_id (latest page wins)
    let mut seen = HashSet::new();
    let deduped: Vec<&RmpProfessor> = professors
        .iter()
        .rev()
        .filter(|p| seen.insert(p.legacy_id))
        .collect();

    let legacy_ids: Vec<i32> = deduped.iter().map(|p| p.legacy_id).collect();
    let graphql_ids: Vec<&str> = deduped.iter().map(|p| p.graphql_id.as_str()).collect();
    let first_names: Vec<String> = deduped
        .iter()
        .map(|p| p.first_name.trim().to_string())
        .collect();
    let first_name_refs: Vec<&str> = first_names.iter().map(|s| s.as_str()).collect();
    let last_names: Vec<String> = deduped
        .iter()
        .map(|p| p.last_name.trim().to_string())
        .collect();
    let last_name_refs: Vec<&str> = last_names.iter().map(|s| s.as_str()).collect();
    let departments: Vec<Option<&str>> = deduped.iter().map(|p| p.department.as_deref()).collect();
    let avg_ratings: Vec<Option<f32>> = deduped.iter().map(|p| p.avg_rating).collect();
    let avg_difficulties: Vec<Option<f32>> = deduped.iter().map(|p| p.avg_difficulty).collect();
    let num_ratings: Vec<i32> = deduped.iter().map(|p| p.num_ratings).collect();
    let would_take_again_pcts: Vec<Option<f32>> =
        deduped.iter().map(|p| p.would_take_again_pct).collect();

    sqlx::query(
        r#"
        INSERT INTO rmp_professors (
            legacy_id, graphql_id, first_name, last_name, department,
            avg_rating, avg_difficulty, num_ratings, would_take_again_pct,
            last_synced_at
        )
        SELECT
            v.legacy_id, v.graphql_id, v.first_name, v.last_name, v.department,
            v.avg_rating, v.avg_difficulty, v.num_ratings, v.would_take_again_pct,
            NOW()
        FROM UNNEST(
            $1::int4[], $2::text[], $3::text[], $4::text[], $5::text[],
            $6::real[], $7::real[], $8::int4[], $9::real[]
        ) AS v(
            legacy_id, graphql_id, first_name, last_name, department,
            avg_rating, avg_difficulty, num_ratings, would_take_again_pct
        )
        ON CONFLICT (legacy_id)
        DO UPDATE SET
            graphql_id = EXCLUDED.graphql_id,
            first_name = EXCLUDED.first_name,
            last_name = EXCLUDED.last_name,
            department = EXCLUDED.department,
            avg_rating = EXCLUDED.avg_rating,
            avg_difficulty = EXCLUDED.avg_difficulty,
            num_ratings = EXCLUDED.num_ratings,
            would_take_again_pct = EXCLUDED.would_take_again_pct,
            last_synced_at = EXCLUDED.last_synced_at
        "#,
    )
    .bind(&legacy_ids)
    .bind(&graphql_ids)
    .bind(&first_name_refs)
    .bind(&last_name_refs)
    .bind(&departments)
    .bind(&avg_ratings)
    .bind(&avg_difficulties)
    .bind(&num_ratings)
    .bind(&would_take_again_pcts)
    .execute(db_pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to batch upsert RMP professors: {}", e))?;

    Ok(())
}

/// Normalize a name for matching: lowercase, trim, strip trailing periods.
fn normalize(s: &str) -> String {
    s.trim().to_lowercase().trim_end_matches('.').to_string()
}

/// Parse Banner's "Last, First Middle" display name into (last, first) tokens.
///
/// Returns `None` if the format is unparseable (no comma, empty parts).
fn parse_display_name(display_name: &str) -> Option<(String, String)> {
    let (last_part, first_part) = display_name.split_once(',')?;
    let last = normalize(last_part);
    // Take only the first token of the first-name portion to drop middle names/initials.
    let first = normalize(first_part.split_whitespace().next()?);
    if last.is_empty() || first.is_empty() {
        return None;
    }
    Some((last, first))
}

/// Auto-match instructors to RMP professors by normalized name.
///
/// Loads all pending instructors and all RMP professors, then matches in Rust
/// using normalized name comparison. Only assigns a match when exactly one RMP
/// professor matches a given instructor.
pub async fn auto_match_instructors(db_pool: &PgPool) -> Result<u64> {
    // Load pending instructors
    let instructors: Vec<(String, String)> = sqlx::query_as(
        "SELECT banner_id, display_name FROM instructors WHERE rmp_match_status = 'pending'",
    )
    .fetch_all(db_pool)
    .await?;

    if instructors.is_empty() {
        info!(matched = 0, "No pending instructors to match");
        return Ok(0);
    }

    // Load all RMP professors
    let professors: Vec<(i32, String, String)> =
        sqlx::query_as("SELECT legacy_id, first_name, last_name FROM rmp_professors")
            .fetch_all(db_pool)
            .await?;

    // Build a lookup: (normalized_last, normalized_first) -> list of legacy_ids
    let mut rmp_index: HashMap<(String, String), Vec<i32>> = HashMap::new();
    for (legacy_id, first, last) in &professors {
        let key = (normalize(last), normalize(first));
        rmp_index.entry(key).or_default().push(*legacy_id);
    }

    // Match each instructor
    let mut matches: Vec<(i32, String)> = Vec::new(); // (legacy_id, banner_id)
    let mut no_comma = 0u64;
    let mut no_match = 0u64;
    let mut ambiguous = 0u64;

    for (banner_id, display_name) in &instructors {
        let Some((last, first)) = parse_display_name(display_name) else {
            no_comma += 1;
            continue;
        };

        let key = (last, first);
        match rmp_index.get(&key) {
            Some(ids) if ids.len() == 1 => {
                matches.push((ids[0], banner_id.clone()));
            }
            Some(ids) => {
                ambiguous += 1;
                debug!(
                    banner_id,
                    display_name,
                    candidates = ids.len(),
                    "Ambiguous RMP match, skipping"
                );
            }
            None => {
                no_match += 1;
            }
        }
    }

    if no_comma > 0 || ambiguous > 0 {
        warn!(
            total_pending = instructors.len(),
            no_comma,
            no_match,
            ambiguous,
            matched = matches.len(),
            "RMP matching diagnostics"
        );
    }

    // Batch update matches
    if matches.is_empty() {
        info!(matched = 0, "Auto-matched instructors to RMP professors");
        return Ok(0);
    }

    let legacy_ids: Vec<i32> = matches.iter().map(|(id, _)| *id).collect();
    let banner_ids: Vec<&str> = matches.iter().map(|(_, bid)| bid.as_str()).collect();

    let result = sqlx::query(
        r#"
        UPDATE instructors i
        SET
            rmp_legacy_id = m.legacy_id,
            rmp_match_status = 'auto'
        FROM UNNEST($1::int4[], $2::text[]) AS m(legacy_id, banner_id)
        WHERE i.banner_id = m.banner_id
        "#,
    )
    .bind(&legacy_ids)
    .bind(&banner_ids)
    .execute(db_pool)
    .await
    .map_err(|e| anyhow::anyhow!("Failed to update instructor RMP matches: {}", e))?;

    let matched = result.rows_affected();
    info!(matched, "Auto-matched instructors to RMP professors");
    Ok(matched)
}

/// Retrieve RMP rating data for an instructor by banner_id.
///
/// Returns `(avg_rating, num_ratings)` if the instructor has an RMP match.
#[allow(dead_code)]
pub async fn get_instructor_rmp_data(
    db_pool: &PgPool,
    banner_id: &str,
) -> Result<Option<(f32, i32)>> {
    let row: Option<(f32, i32)> = sqlx::query_as(
        r#"
        SELECT rp.avg_rating, rp.num_ratings
        FROM instructors i
        JOIN rmp_professors rp ON rp.legacy_id = i.rmp_legacy_id
        WHERE i.banner_id = $1
          AND rp.avg_rating IS NOT NULL
        "#,
    )
    .bind(banner_id)
    .fetch_optional(db_pool)
    .await?;
    Ok(row)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_standard_name() {
        assert_eq!(
            parse_display_name("Smith, John"),
            Some(("smith".into(), "john".into()))
        );
    }

    #[test]
    fn parse_name_with_middle() {
        assert_eq!(
            parse_display_name("Smith, John David"),
            Some(("smith".into(), "john".into()))
        );
    }

    #[test]
    fn parse_name_with_middle_initial() {
        assert_eq!(
            parse_display_name("Garcia, Maria L."),
            Some(("garcia".into(), "maria".into()))
        );
    }

    #[test]
    fn parse_name_with_suffix_in_last() {
        // Banner may encode "Jr." as part of the last name.
        // normalize() strips trailing periods so "Jr." becomes "jr".
        assert_eq!(
            parse_display_name("Smith Jr., James"),
            Some(("smith jr".into(), "james".into()))
        );
    }

    #[test]
    fn parse_no_comma_returns_none() {
        assert_eq!(parse_display_name("SingleName"), None);
    }

    #[test]
    fn parse_empty_first_returns_none() {
        assert_eq!(parse_display_name("Smith,"), None);
    }

    #[test]
    fn parse_empty_last_returns_none() {
        assert_eq!(parse_display_name(", John"), None);
    }

    #[test]
    fn parse_extra_whitespace() {
        assert_eq!(
            parse_display_name("  Doe ,  Jane   Marie  "),
            Some(("doe".into(), "jane".into()))
        );
    }

    #[test]
    fn normalize_trims_and_lowercases() {
        assert_eq!(normalize("  FOO  "), "foo");
    }

    #[test]
    fn normalize_strips_trailing_period() {
        assert_eq!(normalize("Jr."), "jr");
    }
}
