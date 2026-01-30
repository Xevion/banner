//! ISR-style schedule cache for timeline enrollment queries.
//!
//! Loads all courses with their meeting times from the database, parses the
//! JSONB meeting times into a compact in-memory representation, and caches
//! the result. The cache is refreshed in the background every hour using a
//! stale-while-revalidate pattern with singleflight deduplication — readers
//! always get the current cached value instantly, never blocking on a refresh.

use chrono::NaiveDate;
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::watch;
use tracing::{debug, error, info};

/// How often the cache is considered fresh (1 hour).
const REFRESH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(60 * 60);

// ── Compact schedule representation ─────────────────────────────────

/// A single meeting time block, pre-parsed for fast filtering.
#[derive(Debug, Clone)]
pub(crate) struct ParsedSchedule {
    /// Bitmask of days: bit 0 = Monday, bit 6 = Sunday.
    days: u8,
    /// Minutes since midnight for start (e.g. 600 = 10:00).
    begin_minutes: u16,
    /// Minutes since midnight for end (e.g. 650 = 10:50).
    end_minutes: u16,
    /// First day the meeting pattern is active.
    start_date: NaiveDate,
    /// Last day the meeting pattern is active.
    end_date: NaiveDate,
}

/// A course with its enrollment and pre-parsed schedule blocks.
#[derive(Debug, Clone)]
pub(crate) struct CachedCourse {
    pub(crate) subject: String,
    pub(crate) enrollment: i32,
    pub(crate) schedules: Vec<ParsedSchedule>,
}

/// The immutable snapshot of all courses, swapped atomically on refresh.
#[derive(Debug, Clone)]
pub(crate) struct ScheduleSnapshot {
    pub(crate) courses: Vec<CachedCourse>,
    refreshed_at: std::time::Instant,
}

// ── Cache handle ────────────────────────────────────────────────────

/// Shared schedule cache. Clone-cheap (all `Arc`-wrapped internals).
#[derive(Clone)]
pub struct ScheduleCache {
    /// Current snapshot, updated via `watch` channel for lock-free reads.
    rx: watch::Receiver<Arc<ScheduleSnapshot>>,
    /// Sender side, held to push new snapshots.
    tx: Arc<watch::Sender<Arc<ScheduleSnapshot>>>,
    /// Singleflight guard — true while a refresh task is in flight.
    refreshing: Arc<AtomicBool>,
    /// Database pool for refresh queries.
    pool: PgPool,
}

impl ScheduleCache {
    /// Create a new cache with an empty initial snapshot.
    pub(crate) fn new(pool: PgPool) -> Self {
        let empty = Arc::new(ScheduleSnapshot {
            courses: Vec::new(),
            refreshed_at: std::time::Instant::now(),
        });
        let (tx, rx) = watch::channel(empty);
        Self {
            rx,
            tx: Arc::new(tx),
            refreshing: Arc::new(AtomicBool::new(false)),
            pool,
        }
    }

    /// Get the current snapshot. Never blocks on refresh.
    pub(crate) fn snapshot(&self) -> Arc<ScheduleSnapshot> {
        self.rx.borrow().clone()
    }

    /// Check freshness and trigger a background refresh if stale.
    /// Always returns immediately — the caller uses the current snapshot.
    pub(crate) fn ensure_fresh(&self) {
        let snap = self.rx.borrow();
        if snap.refreshed_at.elapsed() < REFRESH_INTERVAL {
            return;
        }
        // Singleflight: only one refresh at a time.
        if self
            .refreshing
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            debug!("Schedule cache refresh already in flight, skipping");
            return;
        }
        let cache = self.clone();
        tokio::spawn(async move {
            match load_snapshot(&cache.pool).await {
                Ok(snap) => {
                    let count = snap.courses.len();
                    let _ = cache.tx.send(Arc::new(snap));
                    info!(courses = count, "Schedule cache refreshed");
                }
                Err(e) => {
                    error!(error = %e, "Failed to refresh schedule cache");
                }
            }
            cache.refreshing.store(false, Ordering::Release);
        });
    }

    /// Force an initial load (blocking). Call once at startup.
    pub(crate) async fn load(&self) -> anyhow::Result<()> {
        let snap = load_snapshot(&self.pool).await?;
        let count = snap.courses.len();
        let _ = self.tx.send(Arc::new(snap));
        info!(courses = count, "Schedule cache initially loaded");
        Ok(())
    }
}

// ── Database loading ────────────────────────────────────────────────

/// Row returned from the lightweight schedule query.
#[derive(sqlx::FromRow)]
struct ScheduleRow {
    subject: String,
    enrollment: i32,
    meeting_times: Value,
}

/// Load all courses and parse their meeting times into a snapshot.
async fn load_snapshot(pool: &PgPool) -> anyhow::Result<ScheduleSnapshot> {
    let start = std::time::Instant::now();

    let rows: Vec<ScheduleRow> =
        sqlx::query_as("SELECT subject, enrollment, meeting_times FROM courses")
            .fetch_all(pool)
            .await?;

    let courses: Vec<CachedCourse> = rows
        .into_iter()
        .map(|row| {
            let schedules = parse_meeting_times(&row.meeting_times);
            CachedCourse {
                subject: row.subject,
                enrollment: row.enrollment,
                schedules,
            }
        })
        .collect();

    debug!(
        courses = courses.len(),
        elapsed_ms = start.elapsed().as_millis(),
        "Schedule snapshot built"
    );

    Ok(ScheduleSnapshot {
        courses,
        refreshed_at: std::time::Instant::now(),
    })
}

// ── Meeting time parsing ────────────────────────────────────────────

/// Parse the JSONB `meeting_times` array into compact `ParsedSchedule` values.
fn parse_meeting_times(value: &Value) -> Vec<ParsedSchedule> {
    let Value::Array(arr) = value else {
        return Vec::new();
    };

    arr.iter().filter_map(parse_one_meeting).collect()
}

fn parse_one_meeting(mt: &Value) -> Option<ParsedSchedule> {
    let begin_time = mt.get("begin_time")?.as_str()?;
    let end_time = mt.get("end_time")?.as_str()?;

    let begin_minutes = parse_hhmm(begin_time)?;
    let end_minutes = parse_hhmm(end_time)?;

    if end_minutes <= begin_minutes {
        return None;
    }

    let start_date = parse_date(mt.get("start_date")?.as_str()?)?;
    let end_date = parse_date(mt.get("end_date")?.as_str()?)?;

    const DAY_KEYS: [&str; 7] = [
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
        "sunday",
    ];
    let mut days: u8 = 0;
    for (bit, key) in DAY_KEYS.iter().enumerate() {
        if mt.get(*key).and_then(Value::as_bool).unwrap_or(false) {
            days |= 1 << bit;
        }
    }

    // Skip meetings with no days (online async, etc.)
    if days == 0 {
        return None;
    }

    Some(ParsedSchedule {
        days,
        begin_minutes,
        end_minutes,
        start_date,
        end_date,
    })
}

/// Parse "HHMM" → minutes since midnight.
fn parse_hhmm(s: &str) -> Option<u16> {
    if s.len() != 4 {
        return None;
    }
    let hours: u16 = s[..2].parse().ok()?;
    let mins: u16 = s[2..].parse().ok()?;
    if hours >= 24 || mins >= 60 {
        return None;
    }
    Some(hours * 60 + mins)
}

/// Parse "MM/DD/YYYY" → NaiveDate.
fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%m/%d/%Y").ok()
}

// ── Slot matching ───────────────────────────────────────────────────

/// Day-of-week as our bitmask index (Monday = 0 .. Sunday = 6).
/// Chrono's `weekday().num_days_from_monday()` already gives 0=Mon..6=Sun.
pub(crate) fn weekday_bit(day: chrono::Weekday) -> u8 {
    1 << day.num_days_from_monday()
}

impl ParsedSchedule {
    /// Check if this schedule is active during a given slot.
    ///
    /// `slot_date` is the calendar date of the slot.
    /// `slot_start` / `slot_end` are minutes since midnight for the 15-min window.
    #[inline]
    pub(crate) fn active_during(
        &self,
        slot_date: NaiveDate,
        slot_weekday_bit: u8,
        slot_start_minutes: u16,
        slot_end_minutes: u16,
    ) -> bool {
        // Day-of-week check
        if self.days & slot_weekday_bit == 0 {
            return false;
        }
        // Date range check
        if slot_date < self.start_date || slot_date > self.end_date {
            return false;
        }
        // Time overlap: meeting [begin, end) overlaps slot [start, end)
        self.begin_minutes < slot_end_minutes && self.end_minutes > slot_start_minutes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use serde_json::json;

    #[test]
    fn parse_hhmm_valid() {
        assert_eq!(parse_hhmm("0000"), Some(0));
        assert_eq!(parse_hhmm("0930"), Some(570));
        assert_eq!(parse_hhmm("1350"), Some(830));
        assert_eq!(parse_hhmm("2359"), Some(1439));
    }

    #[test]
    fn parse_hhmm_invalid() {
        assert_eq!(parse_hhmm(""), None);
        assert_eq!(parse_hhmm("abc"), None);
        assert_eq!(parse_hhmm("2500"), None);
        assert_eq!(parse_hhmm("0060"), None);
    }

    #[test]
    fn parse_date_valid() {
        assert_eq!(
            parse_date("08/26/2025"),
            Some(NaiveDate::from_ymd_opt(2025, 8, 26).unwrap())
        );
    }

    #[test]
    fn parse_meeting_times_basic() {
        let json = json!([{
            "begin_time": "1000",
            "end_time": "1050",
            "start_date": "08/26/2025",
            "end_date": "12/13/2025",
            "monday": true,
            "tuesday": false,
            "wednesday": true,
            "thursday": false,
            "friday": true,
            "saturday": false,
            "sunday": false,
            "building": "NPB",
            "building_description": "North Paseo Building",
            "room": "1.238",
            "campus": "11",
            "meeting_type": "FF",
            "meeting_schedule_type": "AFF"
        }]);

        let schedules = parse_meeting_times(&json);
        assert_eq!(schedules.len(), 1);

        let s = &schedules[0];
        assert_eq!(s.begin_minutes, 600); // 10:00
        assert_eq!(s.end_minutes, 650); // 10:50
        assert_eq!(s.days, 0b0010101); // Mon, Wed, Fri
    }

    #[test]
    fn parse_meeting_times_skips_null_times() {
        let json = json!([{
            "begin_time": null,
            "end_time": null,
            "start_date": "08/26/2025",
            "end_date": "12/13/2025",
            "monday": false,
            "tuesday": false,
            "wednesday": false,
            "thursday": false,
            "friday": false,
            "saturday": false,
            "sunday": false,
            "meeting_type": "OS",
            "meeting_schedule_type": "AFF"
        }]);

        let schedules = parse_meeting_times(&json);
        assert!(schedules.is_empty());
    }

    #[test]
    fn active_during_matching_slot() {
        let sched = ParsedSchedule {
            days: 0b0000001, // Monday
            begin_minutes: 600,
            end_minutes: 650,
            start_date: NaiveDate::from_ymd_opt(2025, 8, 26).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 12, 13).unwrap(),
        };

        // Monday Sept 1 2025, 10:00-10:15 slot
        let date = NaiveDate::from_ymd_opt(2025, 9, 1).unwrap();
        assert!(sched.active_during(date, weekday_bit(chrono::Weekday::Mon), 600, 615));
    }

    #[test]
    fn active_during_wrong_day() {
        let sched = ParsedSchedule {
            days: 0b0000001, // Monday only
            begin_minutes: 600,
            end_minutes: 650,
            start_date: NaiveDate::from_ymd_opt(2025, 8, 26).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 12, 13).unwrap(),
        };

        // Tuesday Sept 2 2025
        let date = NaiveDate::from_ymd_opt(2025, 9, 2).unwrap();
        assert!(!sched.active_during(date, weekday_bit(chrono::Weekday::Tue), 600, 615));
    }

    #[test]
    fn active_during_no_time_overlap() {
        let sched = ParsedSchedule {
            days: 0b0000001,
            begin_minutes: 600, // 10:00
            end_minutes: 650,   // 10:50
            start_date: NaiveDate::from_ymd_opt(2025, 8, 26).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 12, 13).unwrap(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 9, 1).unwrap(); // Monday
        // Slot 11:00-11:15 — after the meeting ends
        assert!(!sched.active_during(date, weekday_bit(chrono::Weekday::Mon), 660, 675));
        // Slot 9:45-10:00 — just before meeting starts (end=600, begin=600 → no overlap)
        assert!(!sched.active_during(date, weekday_bit(chrono::Weekday::Mon), 585, 600));
    }

    #[test]
    fn active_during_outside_date_range() {
        let sched = ParsedSchedule {
            days: 0b0000001,
            begin_minutes: 600,
            end_minutes: 650,
            start_date: NaiveDate::from_ymd_opt(2025, 8, 26).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 12, 13).unwrap(),
        };

        // Monday Jan 6 2025 — before semester
        let date = NaiveDate::from_ymd_opt(2025, 1, 6).unwrap();
        assert!(!sched.active_during(date, weekday_bit(chrono::Weekday::Mon), 600, 615));
    }

    #[test]
    fn active_during_edge_overlap() {
        let sched = ParsedSchedule {
            days: 0b0000001,
            begin_minutes: 600,
            end_minutes: 650,
            start_date: NaiveDate::from_ymd_opt(2025, 8, 26).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 12, 13).unwrap(),
        };

        let date = NaiveDate::from_ymd_opt(2025, 9, 1).unwrap();
        // Slot 10:45-11:00 — overlaps last 5 minutes of meeting
        assert!(sched.active_during(date, weekday_bit(chrono::Weekday::Mon), 645, 660));
        // Slot 9:45-10:00 — ends exactly when meeting starts, no overlap
        assert!(!sched.active_during(date, weekday_bit(chrono::Weekday::Mon), 585, 600));
        // Slot 10:50-11:05 — starts exactly when meeting ends, no overlap
        assert!(!sched.active_during(date, weekday_bit(chrono::Weekday::Mon), 650, 665));
    }
}
