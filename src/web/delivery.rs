//! Delivery mode classification for course sections.
//!
//! Moves the delivery concern logic (previously in the TypeScript frontend)
//! to the backend so it ships as part of the API response.

use crate::data::models::DbMeetingTime;
use serde::Serialize;
use ts_rs::TS;

/// Banner instructional method codes for fully-online delivery.
const ONLINE_METHODS: &[&str] = &["OA", "OS", "OH"];

/// Banner instructional method codes for hybrid delivery.
const HYBRID_METHODS: &[&str] = &["HB", "H1", "H2"];

/// Banner campus code for the main (San Antonio) campus.
const MAIN_CAMPUS: &str = "11";

/// Banner campus codes that represent online/virtual campuses.
const ONLINE_CAMPUSES: &[&str] = &["9", "ONL"];

/// Delivery mode classification for visual accents on location cells.
///
/// `None` means normal in-person on the main campus (no accent needed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum DeliveryMode {
    /// Fully online with no physical location (OA, OS, OH without INT building).
    Online,
    /// Internet campus with INT building code.
    Internet,
    /// Mix of online and in-person (HB, H1, H2).
    Hybrid,
    /// In-person but not on Main Campus.
    OffCampus,
}

/// Classify the delivery mode for a course section.
///
/// Returns `None` for normal in-person sections on the main campus.
pub fn classify_delivery_mode(
    instructional_method: Option<&str>,
    campus: Option<&str>,
    meeting_times: &[DbMeetingTime],
) -> Option<DeliveryMode> {
    if let Some(method) = instructional_method {
        if ONLINE_METHODS.contains(&method) {
            let has_int_building = meeting_times.iter().any(|mt| {
                mt.location.as_ref().and_then(|loc| loc.building.as_deref()) == Some("INT")
            });
            return Some(if has_int_building {
                DeliveryMode::Internet
            } else {
                DeliveryMode::Online
            });
        }
        if HYBRID_METHODS.contains(&method) {
            return Some(DeliveryMode::Hybrid);
        }
    }

    if let Some(campus) = campus
        && campus != MAIN_CAMPUS
        && !ONLINE_CAMPUSES.contains(&campus)
    {
        return Some(DeliveryMode::OffCampus);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::course_types::{DateRange, MeetingLocation};
    use chrono::NaiveDate;
    use std::collections::BTreeSet;

    fn make_mt(building: Option<&str>) -> DbMeetingTime {
        DbMeetingTime {
            time_range: None,
            date_range: DateRange {
                start: NaiveDate::from_ymd_opt(2024, 8, 26).unwrap(),
                end: NaiveDate::from_ymd_opt(2024, 12, 12).unwrap(),
            },
            days: BTreeSet::new(),
            location: building.map(|b| MeetingLocation {
                building: Some(b.to_string()),
                building_description: None,
                room: None,
                campus: None,
            }),
            meeting_type: "CLAS".to_string(),
            meeting_schedule_type: "LEC".to_string(),
        }
    }

    #[test]
    fn online_without_int_building() {
        for method in &["OA", "OS", "OH"] {
            assert_eq!(
                classify_delivery_mode(Some(method), Some("9"), &[make_mt(None)]),
                Some(DeliveryMode::Online),
            );
        }
    }

    #[test]
    fn online_with_int_building() {
        assert_eq!(
            classify_delivery_mode(Some("OA"), Some("9"), &[make_mt(Some("INT"))]),
            Some(DeliveryMode::Internet),
        );
    }

    #[test]
    fn hybrid_methods() {
        for method in &["HB", "H1", "H2"] {
            assert_eq!(
                classify_delivery_mode(Some(method), Some("11"), &[]),
                Some(DeliveryMode::Hybrid),
            );
        }
    }

    #[test]
    fn off_campus() {
        assert_eq!(
            classify_delivery_mode(None, Some("22"), &[]),
            Some(DeliveryMode::OffCampus),
        );
    }

    #[test]
    fn main_campus_in_person() {
        assert_eq!(classify_delivery_mode(None, Some("11"), &[]), None);
    }

    #[test]
    fn online_campus_no_method_is_normal() {
        // Campus 9 or ONL without an online method → None (no accent)
        assert_eq!(classify_delivery_mode(None, Some("9"), &[]), None);
        assert_eq!(classify_delivery_mode(None, Some("ONL"), &[]), None);
    }

    #[test]
    fn no_method_no_campus() {
        assert_eq!(classify_delivery_mode(None, None, &[]), None);
    }
}
