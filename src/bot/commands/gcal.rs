//! Google Calendar command implementation.

use crate::banner::{Course, MeetingScheduleInfo, TimeRange};
use crate::bot::{Context, Error};
use chrono::NaiveDate;
use std::collections::HashMap;
use tracing::{error, info};
use url::Url;

const TIMESTAMP_FORMAT: &str = "%Y%m%dT%H%M%SZ";

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
    let mut meeting_times = match banner_api.get_course_meeting_time(term, crn).await {
        Ok(meeting_time) => meeting_time,
        Err(e) => {
            error!("Failed to get meeting times: {}", e);
            return Err(Error::from(e));
        }
    };

    struct LinkDetail {
        link: String,
        detail: String,
    }

    let response: Vec<LinkDetail> = match meeting_times.len() {
        0 => Err(anyhow::anyhow!("No meeting times found for this course.")),
        1.. => {
            let links = meeting_times
                .iter()
                .map(|m| {
                    let link = generate_gcal_url(&course, m)?;
                    let detail = match &m.time_range {
                        Some(range) => range.format_12hr(),
                        None => m.days_string(),
                    };
                    Ok(LinkDetail { link, detail })
                })
                .collect::<Result<Vec<LinkDetail>, anyhow::Error>>()?;
            Ok(links)
        }
    }?;

    ctx.say(
        response
            .iter()
            .map(|LinkDetail { link, detail }| {
                format!("[Add to Google Calendar](<{link}>) ({detail})")
            })
            .collect::<Vec<String>>()
            .join("\n"),
    )
    .await?;

    info!("gcal command completed for CRN: {}", crn);
    Ok(())
}

/// Generate Google Calendar URL for a course
fn generate_gcal_url(
    course: &Course,
    meeting_time: &MeetingScheduleInfo,
) -> Result<String, anyhow::Error> {
    // Get start and end dates
    let (start, end) = {
        let central_tz = chrono_tz::US::Central;
        let (start, end) = meeting_time.datetime_range();
        (
            start.with_timezone(&central_tz),
            end.with_timezone(&central_tz),
        )
    };

    // Generate RRULE for recurrence
    let rrule = generate_rrule(meeting_time, end.date_naive());

    // Build calendar URL
    let mut params = HashMap::new();

    let course_text = format!(
        "{} {} - {}",
        course.subject, course.course_number, course.course_title
    );
    let dates_text = format!("{}/{}", start, end);

    // Get instructor name
    let instructor_name = if !course.faculty.is_empty() {
        &course.faculty[0].display_name
    } else {
        "Unknown"
    };

    let days_text = meeting_time.days_string();
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
fn generate_rrule(meeting_time: &MeetingScheduleInfo, end_date: NaiveDate) -> String {
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
