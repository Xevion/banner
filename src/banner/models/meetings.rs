use chrono::{NaiveDate, NaiveTime, Timelike};
use serde::{Deserialize, Serialize};

use super::terms::Term;

/// Represents a faculty member associated with a course.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FacultyItem {
    pub banner_id: u32,               // e.g. 150161
    pub category: Option<String>,     // zero-padded digits
    pub class: String,                // internal class name
    pub course_reference_number: u32, // CRN, e.g. 27294
    pub display_name: String,         // "LastName, FirstName"
    pub email_address: String,        // e.g. FirstName.LastName@utsa.edu
    pub primary_indicator: bool,
    pub term: Term,
}

/// Meeting time information for a course.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeetingTime {
    pub start_date: String,              // MM/DD/YYYY, e.g. 08/26/2025
    pub end_date: String,                // MM/DD/YYYY, e.g. 08/26/2025
    pub begin_time: String,              // HHMM, e.g. 1000
    pub end_time: String,                // HHMM, e.g. 1100
    pub category: String,                // unknown meaning, e.g. 01, 02, etc.
    pub class: String, // internal class name, e.g. net.hedtech.banner.general.overall.MeetingTimeDecorator
    pub monday: bool,  // true if the meeting time occurs on Monday
    pub tuesday: bool, // true if the meeting time occurs on Tuesday
    pub wednesday: bool, // true if the meeting time occurs on Wednesday
    pub thursday: bool, // true if the meeting time occurs on Thursday
    pub friday: bool,  // true if the meeting time occurs on Friday
    pub saturday: bool, // true if the meeting time occurs on Saturday
    pub sunday: bool,  // true if the meeting time occurs on Sunday
    pub room: String,  // e.g. 1.238
    pub term: Term,    // e.g. 202510
    pub building: String, // e.g. NPB
    pub building_description: String, // e.g. North Paseo Building
    pub campus: String, // campus code, e.g. 11
    pub campus_description: String, // name of campus, e.g. Main Campus
    pub course_reference_number: String, // CRN, e.g. 27294
    pub credit_hour_session: f64, // e.g. 3.0
    pub hours_week: f64, // e.g. 3.0
    pub meeting_schedule_type: String, // e.g. AFF
    pub meeting_type: String, // e.g. HB, H2, H1, OS, OA, OH, ID, FF
    pub meeting_type_description: String,
}

/// API response wrapper for meeting times.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingTimesApiResponse {
    pub fmt: Vec<MeetingTimeResponse>,
}

/// Meeting time response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MeetingTimeResponse {
    pub category: Option<String>,
    pub class: String,
    pub course_reference_number: String,
    pub faculty: Vec<FacultyItem>,
    pub meeting_time: MeetingTime,
    pub term: String,
}

impl MeetingTimeResponse {
    /// Returns a formatted string representation of the meeting time.
    pub fn to_string(&self) -> String {
        match self.meeting_time.meeting_type.as_str() {
            "HB" | "H2" | "H1" => format!("{}\nHybrid {}", self.time_string(), self.place_string()),
            "OS" => format!("{}\nOnline Only", self.time_string()),
            "OA" => "No Time\nOnline Asynchronous".to_string(),
            "OH" => format!("{}\nOnline Partial", self.time_string()),
            "ID" => "To Be Arranged".to_string(),
            "FF" => format!("{}\n{}", self.time_string(), self.place_string()),
            _ => "Unknown".to_string(),
        }
    }

    /// Returns a formatted string of the meeting times.
    pub fn time_string(&self) -> String {
        let start_time = self.parse_time(&self.meeting_time.begin_time);
        let end_time = self.parse_time(&self.meeting_time.end_time);

        match (start_time, end_time) {
            (Some(start), Some(end)) => {
                format!(
                    "{} {}-{}",
                    self.days_string(),
                    format_time(start),
                    format_time(end)
                )
            }
            _ => "???".to_string(),
        }
    }

    /// Returns a formatted string representing the location of the meeting.
    pub fn place_string(&self) -> String {
        if self.meeting_time.room.is_empty() {
            "Online".to_string()
        } else {
            format!(
                "{} | {} | {} {}",
                self.meeting_time.campus_description,
                self.meeting_time.building_description,
                self.meeting_time.building,
                self.meeting_time.room
            )
        }
    }

    /// Returns a compact string representation of meeting days.
    pub fn days_string(&self) -> String {
        let mut days = String::new();
        if self.meeting_time.monday {
            days.push('M');
        }
        if self.meeting_time.tuesday {
            days.push_str("Tu");
        }
        if self.meeting_time.wednesday {
            days.push('W');
        }
        if self.meeting_time.thursday {
            days.push_str("Th");
        }
        if self.meeting_time.friday {
            days.push('F');
        }
        if self.meeting_time.saturday {
            days.push_str("Sa");
        }
        if self.meeting_time.sunday {
            days.push_str("Su");
        }

        if days.is_empty() {
            "None".to_string()
        } else if days.len() == 14 {
            // All days
            "Everyday".to_string()
        } else {
            days
        }
    }

    /// Parse a time string in HHMM format to NaiveTime.
    fn parse_time(&self, time_str: &str) -> Option<NaiveTime> {
        if time_str.is_empty() {
            return None;
        }

        let time_int: u32 = time_str.parse().ok()?;
        let hours = time_int / 100;
        let minutes = time_int % 100;

        NaiveTime::from_hms_opt(hours, minutes, 0)
    }

    /// Parse a date string in MM/DD/YYYY format.
    pub fn parse_date(date_str: &str) -> Option<NaiveDate> {
        NaiveDate::parse_from_str(date_str, "%m/%d/%Y").ok()
    }

    /// Get the start date as NaiveDate.
    pub fn start_date(&self) -> Option<NaiveDate> {
        Self::parse_date(&self.meeting_time.start_date)
    }

    /// Get the end date as NaiveDate.
    pub fn end_date(&self) -> Option<NaiveDate> {
        Self::parse_date(&self.meeting_time.end_date)
    }
}

/// Format a NaiveTime in 12-hour format.
fn format_time(time: NaiveTime) -> String {
    let hour = time.hour();
    let minute = time.minute();

    if hour == 0 {
        format!("12:{:02}AM", minute)
    } else if hour < 12 {
        format!("{}:{:02}AM", hour, minute)
    } else if hour == 12 {
        format!("12:{:02}PM", minute)
    } else {
        format!("{}:{:02}PM", hour - 12, minute)
    }
}
