//! Google Calendar command implementation.

use crate::banner::{Course, DayOfWeek, MeetingScheduleInfo, Term};
use crate::bot::{Context, Error};
use chrono::NaiveDate;
use std::collections::HashMap;
use tracing::{error, info};
use url::Url;

/// Generate a link to create a Google Calendar event for a course
#[poise::command(slash_command)]
pub async fn gcal(
    ctx: Context<'_>,
    #[description = "Course Reference Number (CRN)"] crn: i32,
) -> Result<(), Error> {
    let user = ctx.author();
    info!(source = user.name, target = crn, "gcal command invoked");

    ctx.defer().await?;

    let app_state = &ctx.data().app_state;
    let banner_api = &app_state.banner_api;

    // Get current term dynamically
    let current_term_status = Term::get_current();
    let term = current_term_status.inner();

    // Fetch live course data from Redis cache via AppState
    let course = match app_state
        .get_course_or_fetch(&term.to_string(), &crn.to_string())
        .await
    {
        Ok(course) => course,
        Err(e) => {
            error!(%e, crn, "Failed to fetch course data");
            return Err(Error::from(e));
        }
    };

    // Get meeting times
    let meeting_times = match banner_api
        .get_course_meeting_time(&term.to_string(), crn)
        .await
    {
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
            // Sort meeting times by start time of their TimeRange
            let mut sorted_meeting_times = meeting_times.to_vec();
            sorted_meeting_times.sort_unstable_by(|a, b| {
                // Primary sort: by start time
                match (&a.time_range, &b.time_range) {
                    (Some(a_time), Some(b_time)) => a_time.start.cmp(&b_time.start),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.days.bits().cmp(&b.days.bits()),
                }
            });

            let links = sorted_meeting_times
                .iter()
                .map(|m| {
                    let link = generate_gcal_url(&course, m)?;
                    let detail = match &m.time_range {
                        Some(range) => format!("{} {}", m.days_string(), range.format_12hr()),
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
    let course_text = format!(
        "{} {} - {}",
        course.subject, course.course_number, course.course_title
    );

    let dates_text = {
        let (start, end) = meeting_time.datetime_range();
        format!(
            "{}/{}",
            start.format("%Y%m%dT%H%M%S"),
            end.format("%Y%m%dT%H%M%S")
        )
    };

    // Get instructor name
    let instructor_name = if !course.faculty.is_empty() {
        &course.faculty[0].display_name
    } else {
        "Unknown"
    };

    // The event description
    let details_text = format!(
        "CRN: {}\nInstructor: {}\nDays: {}",
        course.course_reference_number,
        instructor_name,
        meeting_time.days_string()
    );

    // The event location
    let location_text = meeting_time.place_string();

    // The event recurrence rule
    let recur_text = generate_rrule(meeting_time, meeting_time.date_range.end);

    let mut params = HashMap::new();
    params.insert("action", "TEMPLATE");
    params.insert("text", &course_text);
    params.insert("dates", &dates_text);
    params.insert("details", &details_text);
    params.insert("location", &location_text);
    params.insert("trp", "true");
    params.insert("ctz", "America/Chicago");
    params.insert("recur", &recur_text);

    Ok(Url::parse_with_params("https://calendar.google.com/calendar/render", &params)?.to_string())
}

/// Generate RRULE for recurrence
fn generate_rrule(meeting_time: &MeetingScheduleInfo, end_date: NaiveDate) -> String {
    let days_of_week = meeting_time.days_of_week();
    let by_day = days_of_week
        .iter()
        .map(|day| match day {
            DayOfWeek::Monday => "MO",
            DayOfWeek::Tuesday => "TU",
            DayOfWeek::Wednesday => "WE",
            DayOfWeek::Thursday => "TH",
            DayOfWeek::Friday => "FR",
            DayOfWeek::Saturday => "SA",
            DayOfWeek::Sunday => "SU",
        })
        .collect::<Vec<&str>>()
        .join(",");

    // Format end date for RRULE (YYYYMMDD format)
    let until = end_date.format("%Y%m%dT000000Z").to_string();

    format!("RRULE:FREQ=WEEKLY;BYDAY={by_day};UNTIL={until}")
}
