//! Timeline API endpoint for enrollment aggregation by subject over time.
//!
//! Accepts multiple time ranges, merges overlaps, aligns to 15-minute
//! slot boundaries, and returns per-subject enrollment totals for each slot.
//! Only courses whose meeting times overlap a given slot contribute to that
//! slot's totals — so the chart reflects the actual class schedule rhythm.
//!
//! Course data is served from an ISR-style in-memory cache (see
//! [`ScheduleCache`]) that refreshes hourly in the background with
//! stale-while-revalidate semantics.

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use chrono::{DateTime, Datelike, Duration, NaiveTime, Timelike, Utc};
use chrono_tz::US::Central;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use ts_rs::TS;

use crate::state::AppState;
use crate::web::schedule_cache::weekday_bit;

/// 15 minutes in seconds, matching the frontend `SLOT_INTERVAL_MS`.
const SLOT_SECONDS: i64 = 15 * 60;
const SLOT_MINUTES: u16 = 15;

/// Maximum number of ranges in a single request.
const MAX_RANGES: usize = 20;

/// Maximum span of a single range (72 hours).
const MAX_RANGE_SPAN: Duration = Duration::hours(72);

/// Maximum total span across all ranges to prevent excessive queries.
const MAX_TOTAL_SPAN: Duration = Duration::hours(168); // 1 week

// ── Request / Response types ────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimelineRequest {
    ranges: Vec<TimeRange>,
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimeRange {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimelineResponse {
    /// 15-minute slots with per-subject enrollment totals, sorted by time.
    slots: Vec<TimelineSlot>,
    /// All subject codes present in the returned data.
    subjects: Vec<String>,
}

#[derive(Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TimelineSlot {
    /// ISO-8601 timestamp at the start of this 15-minute bucket.
    time: DateTime<Utc>,
    /// Subject code → total enrollment in this slot.
    #[ts(type = "Record<string, number>")]
    subjects: BTreeMap<String, i64>,
}

// ── Error type ──────────────────────────────────────────────────────

pub(crate) struct TimelineError {
    status: StatusCode,
    message: String,
}

impl TimelineError {
    fn bad_request(msg: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: msg.into(),
        }
    }
}

impl IntoResponse for TimelineError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(serde_json::json!({ "error": self.message })),
        )
            .into_response()
    }
}

// ── Alignment helpers ───────────────────────────────────────────────

/// Floor a timestamp to the nearest 15-minute boundary.
fn align_floor(ts: DateTime<Utc>) -> DateTime<Utc> {
    let secs = ts.timestamp();
    let aligned = (secs / SLOT_SECONDS) * SLOT_SECONDS;
    DateTime::from_timestamp(aligned, 0).unwrap_or(ts)
}

/// Ceil a timestamp to the nearest 15-minute boundary.
fn align_ceil(ts: DateTime<Utc>) -> DateTime<Utc> {
    let secs = ts.timestamp();
    let aligned = ((secs + SLOT_SECONDS - 1) / SLOT_SECONDS) * SLOT_SECONDS;
    DateTime::from_timestamp(aligned, 0).unwrap_or(ts)
}

// ── Range merging ───────────────────────────────────────────────────

/// Aligned, validated range.
#[derive(Debug, Clone, Copy)]
struct AlignedRange {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

/// Merge overlapping/adjacent ranges into a minimal set.
fn merge_ranges(mut ranges: Vec<AlignedRange>) -> Vec<AlignedRange> {
    if ranges.is_empty() {
        return ranges;
    }
    ranges.sort_by_key(|r| r.start);
    let mut merged: Vec<AlignedRange> = vec![ranges[0]];
    for r in &ranges[1..] {
        let last = merged.last_mut().unwrap();
        if r.start <= last.end {
            last.end = last.end.max(r.end);
        } else {
            merged.push(*r);
        }
    }
    merged
}

/// Generate all aligned slot timestamps within the merged ranges.
fn generate_slots(merged: &[AlignedRange]) -> BTreeSet<DateTime<Utc>> {
    let mut slots = BTreeSet::new();
    for range in merged {
        let mut t = range.start;
        while t < range.end {
            slots.insert(t);
            t += Duration::seconds(SLOT_SECONDS);
        }
    }
    slots
}

// ── Handler ─────────────────────────────────────────────────────────

/// `POST /api/timeline`
///
/// Accepts a JSON body with multiple time ranges. Returns per-subject
/// enrollment totals bucketed into 15-minute slots. Only courses whose
/// meeting schedule overlaps a slot contribute to that slot's count.
pub(crate) async fn timeline(
    State(state): State<AppState>,
    Json(body): Json<TimelineRequest>,
) -> Result<Json<TimelineResponse>, TimelineError> {
    // ── Validate ────────────────────────────────────────────────────
    if body.ranges.is_empty() {
        return Err(TimelineError::bad_request("At least one range is required"));
    }
    if body.ranges.len() > MAX_RANGES {
        return Err(TimelineError::bad_request(format!(
            "Too many ranges (max {MAX_RANGES})"
        )));
    }

    let mut aligned: Vec<AlignedRange> = Vec::with_capacity(body.ranges.len());
    for r in &body.ranges {
        if r.end <= r.start {
            return Err(TimelineError::bad_request(format!(
                "Range end ({}) must be after start ({})",
                r.end, r.start
            )));
        }
        let span = r.end - r.start;
        if span > MAX_RANGE_SPAN {
            return Err(TimelineError::bad_request(format!(
                "Range span ({} hours) exceeds maximum ({} hours)",
                span.num_hours(),
                MAX_RANGE_SPAN.num_hours()
            )));
        }
        aligned.push(AlignedRange {
            start: align_floor(r.start),
            end: align_ceil(r.end),
        });
    }

    let merged = merge_ranges(aligned);

    // Validate total span
    let total_span: Duration = merged.iter().map(|r| r.end - r.start).sum();
    if total_span > MAX_TOTAL_SPAN {
        return Err(TimelineError::bad_request(format!(
            "Total time span ({} hours) exceeds maximum ({} hours)",
            total_span.num_hours(),
            MAX_TOTAL_SPAN.num_hours()
        )));
    }

    // ── Get cached schedule data (ISR: stale-while-revalidate) ───────
    state.schedule_cache.ensure_fresh();
    let snapshot = state.schedule_cache.snapshot();

    // ── Build per-slot enrollment by filtering on meeting times ──────
    let slot_times = generate_slots(&merged);
    let mut all_subjects: BTreeSet<String> = BTreeSet::new();

    let slots: Vec<TimelineSlot> = slot_times
        .into_iter()
        .map(|utc_time| {
            // Convert UTC slot to Central time for local day-of-week and time-of-day
            let local = utc_time.with_timezone(&Central);
            let local_date = local.date_naive();
            let local_time = local.time();
            let weekday = local.weekday();
            let wday_bit = weekday_bit(weekday);
            let slot_start_minutes = time_to_minutes(local_time);
            let slot_end_minutes = slot_start_minutes + SLOT_MINUTES;

            let mut subject_totals: BTreeMap<String, i64> = BTreeMap::new();

            for course in &snapshot.courses {
                let active = course.schedules.iter().any(|s| {
                    s.active_during(local_date, wday_bit, slot_start_minutes, slot_end_minutes)
                });
                if active {
                    *subject_totals.entry(course.subject.clone()).or_default() +=
                        course.enrollment as i64;
                }
            }

            all_subjects.extend(subject_totals.keys().cloned());

            TimelineSlot {
                time: utc_time,
                subjects: subject_totals,
            }
        })
        .collect();

    let subjects: Vec<String> = all_subjects.into_iter().collect();

    Ok(Json(TimelineResponse { slots, subjects }))
}

/// Convert a `NaiveTime` to minutes since midnight.
fn time_to_minutes(t: NaiveTime) -> u16 {
    (t.hour() * 60 + t.minute()) as u16
}
