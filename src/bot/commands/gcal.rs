//! Google Calendar command implementation.

use crate::banner::{Course, MeetingTime, MeetingTimeResponse};
use crate::bot::{Context, Error};
use chrono::{Datelike, NaiveDate, NaiveTime, TimeZone, Timelike, Utc};
use std::collections::HashMap;
use tracing::{error, info};
use url::Url;

/// Generate a link to create a Google Calendar event for a course
#[poise::command(slash_command, prefix_command)]
pub async fn gcal(
    ctx: Context<'_>,
    #[description = "Course Reference Number (CRN)"] crn: i32,
) -> Result<(), Error> {
    let user = ctx.author();
    info!(source = user.name, target = crn, "gcal command invoked");

    ctx.defer().await?;

    let app_state = &ctx.data().app_state;
    let banner_api = &app_state.banner_api;

    // TODO: Get current term dynamically
    let term = 202610; // Hardcoded for now

    // TODO: Replace with actual course data when BannerApi::get_course is implemented
    let course = Course {
        id: 0,
        term: term.to_string(),
        term_desc: "Fall 2026".to_string(),
        course_reference_number: crn.to_string(),
        part_of_term: "1".to_string(),
        course_number: "0000".to_string(),
        subject: "CS".to_string(),
        subject_description: "Computer Science".to_string(),
        sequence_number: "001".to_string(),
        campus_description: "Main Campus".to_string(),
        schedule_type_description: "Lecture".to_string(),
        course_title: "Example Course".to_string(),
        credit_hours: 3,
        maximum_enrollment: 30,
        enrollment: 25,
        seats_available: 5,
        wait_capacity: 10,
        wait_count: 0,
        cross_list: None,
        cross_list_capacity: None,
        cross_list_count: None,
        cross_list_available: None,
        credit_hour_high: None,
        credit_hour_low: None,
        credit_hour_indicator: None,
        open_section: true,
        link_identifier: None,
        is_section_linked: false,
        subject_course: "CS0000".to_string(),
        reserved_seat_summary: None,
        instructional_method: "FF".to_string(),
        instructional_method_description: "Face to Face".to_string(),
        section_attributes: vec![],
        faculty: vec![],
        meetings_faculty: vec![],
    };

    // Get meeting times
    let meeting_times = banner_api.get_course_meeting_time(term, crn).await?;

    if meeting_times.is_empty() {
        ctx.say("No meeting times found for this course").await?;
        return Ok(());
    }

    // Find a meeting time that actually meets (not ID or OA types)
    let meeting_time = meeting_times
        .iter()
        .find(|mt| !matches!(mt.meeting_time.meeting_type.as_str(), "ID" | "OA"))
        .ok_or("Course does not meet at a defined moment in time")?;

    // Generate the Google Calendar URL
    match generate_gcal_url(&course, meeting_time) {
        Ok(calendar_url) => {
            ctx.say(format!("[Add to Google Calendar](<{}>)", calendar_url))
                .await?;
        }
        Err(e) => {
            error!("Failed to generate Google Calendar URL: {}", e);
            ctx.say(format!("Error generating calendar link: {}", e))
                .await?;
        }
    }

    info!("gcal command completed for CRN: {}", crn);
    Ok(())
}

/// Generate Google Calendar URL for a course
fn generate_gcal_url(course: &Course, meeting_time: &MeetingTimeResponse) -> Result<String, Error> {
    // Get start and end dates
    let start_date = meeting_time
        .start_date()
        .ok_or("Could not parse start date")?;
    let end_date = meeting_time.end_date().ok_or("Could not parse end date")?;

    // Get start and end times - parse from the time string
    let time_str = meeting_time.time_string();
    let time_parts: Vec<&str> = time_str.split(' ').collect();

    if time_parts.len() < 2 {
        return Err(format!(
            "Invalid time format: expected at least 2 parts, got {} parts. Time string: '{}'",
            time_parts.len(),
            time_str
        )
        .into());
    }

    let time_range = time_parts[1];
    let times: Vec<&str> = time_range.split('-').collect();

    if times.len() != 2 {
        return Err(format!(
            "Invalid time range format: expected 2 parts, got {} parts. Time range: '{}'",
            times.len(),
            time_range
        )
        .into());
    }

    // Create timestamps in UTC (assuming Central time)
    let central_tz = chrono_tz::US::Central;

    let dt_start = central_tz
        .with_ymd_and_hms(
            start_date.year(),
            start_date.month(),
            start_date.day(),
            start_time.hour(),
            start_time.minute(),
            0,
        )
        .unwrap()
        .with_timezone(&Utc);

    let dt_end = central_tz
        .with_ymd_and_hms(
            end_date.year(),
            end_date.month(),
            end_date.day(),
            end_time.hour(),
            end_time.minute(),
            0,
        )
        .unwrap()
        .with_timezone(&Utc);

    // Format times in UTC for Google Calendar
    let start_str = dt_start.format("%Y%m%dT%H%M%SZ").to_string();
    let end_str = dt_end.format("%Y%m%dT%H%M%SZ").to_string();

    // Generate RRULE for recurrence
    let rrule = generate_rrule(meeting_time, end_date);

    // Build calendar URL
    let mut params = HashMap::new();

    let course_text = format!(
        "{} {} - {}",
        course.subject, course.course_number, course.course_title
    );
    let dates_text = format!("{}/{}", start_str, end_str);

    // Get instructor name
    let instructor_name = if !course.faculty.is_empty() {
        &course.faculty[0].display_name
    } else {
        "Unknown"
    };

    let days_text = weekdays_to_string(&meeting_time.meeting_time);
    let details_text = format!(
        "CRN: {}\nInstructor: {}\nDays: {}",
        course.course_reference_number, instructor_name, days_text
    );

    let location_text = meeting_time.place_string();
    let recur_text = format!("RRULE:{}", rrule);

    params.insert("action", "TEMPLATE");
    params.insert("text", &course_text);
    params.insert("dates", &dates_text);
    params.insert("details", &details_text);
    params.insert("location", &location_text);
    params.insert("trp", "true");
    params.insert("ctz", "America/Chicago");
    params.insert("recur", &recur_text);

    // Build URL
    let mut url = Url::parse("https://calendar.google.com/calendar/render")?;
    for (key, value) in params {
        url.query_pairs_mut().append_pair(key, value);
    }

    Ok(url.to_string())
}

/// Generate RRULE for recurrence
fn generate_rrule(meeting_time: &MeetingTimeResponse, end_date: NaiveDate) -> String {
    let by_day = meeting_time.days_string();

    // Handle edge cases where days_string might return "None" or empty
    let by_day = if by_day.is_empty() || by_day == "None" {
        "MO".to_string() // Default to Monday
    } else {
        // Convert our day format to Google Calendar format
        by_day
            .replace("M", "MO")
            .replace("Tu", "TU")
            .replace("W", "WE")
            .replace("Th", "TH")
            .replace("F", "FR")
            .replace("Sa", "SA")
            .replace("Su", "SU")
    };

    // Format end date for RRULE (YYYYMMDD format)
    let until = end_date.format("%Y%m%d").to_string();

    // Build the RRULE string manually to avoid formatting issues
    let mut rrule = String::new();
    rrule.push_str("FREQ=WEEKLY;BYDAY=");
    rrule.push_str(&by_day);
    rrule.push_str(";UNTIL=");
    rrule.push_str(&until);

    rrule
}

/// Parse time from formatted string (e.g., "8:00AM", "12:30PM")
fn parse_time_from_formatted(time_str: &str) -> Result<NaiveTime, Error> {
    let time_str = time_str.trim();

    // Handle 12-hour format: "8:00AM", "12:30PM", etc.
    if time_str.ends_with("AM") || time_str.ends_with("PM") {
        let (time_part, ampm) = time_str.split_at(time_str.len() - 2);
        let parts: Vec<&str> = time_part.split(':').collect();

        if parts.len() != 2 {
            return Err("Invalid time format".into());
        }

        let hour: u32 = parts[0].parse()?;
        let minute: u32 = parts[1].parse()?;

        let adjusted_hour = match ampm {
            "AM" => {
                if hour == 12 {
                    0
                } else {
                    hour
                }
            }
            "PM" => {
                if hour == 12 {
                    12
                } else {
                    hour + 12
                }
            }
            _ => return Err("Invalid AM/PM indicator".into()),
        };

        chrono::NaiveTime::from_hms_opt(adjusted_hour, minute, 0).ok_or("Invalid time".into())
    } else {
        // Handle 24-hour format: "08:00", "13:30"
        let parts: Vec<&str> = time_str.split(':').collect();

        if parts.len() != 2 {
            return Err("Invalid time format".into());
        }

        let hour: u32 = parts[0].parse()?;
        let minute: u32 = parts[1].parse()?;

        NaiveTime::from_hms_opt(hour, minute, 0).ok_or("Invalid time".into())
    }
}

/// Convert weekdays to string representation
fn weekdays_to_string(meeting_time: &MeetingTime) -> String {
    let mut result = String::new();

    if meeting_time.monday {
        result.push_str("M");
    }
    if meeting_time.tuesday {
        result.push_str("Tu");
    }
    if meeting_time.wednesday {
        result.push_str("W");
    }
    if meeting_time.thursday {
        result.push_str("Th");
    }
    if meeting_time.friday {
        result.push_str("F");
    }
    if meeting_time.saturday {
        result.push_str("Sa");
    }
    if meeting_time.sunday {
        result.push_str("Su");
    }

    if result.is_empty() {
        "None".to_string()
    } else if result.len() == 14 {
        // All days
        "Everyday".to_string()
    } else {
        result
    }
}
