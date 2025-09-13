//! ICS command implementation for generating calendar files.

use crate::banner::{Course, MeetingScheduleInfo};
use crate::bot::{Context, Error, utils};
use chrono::Utc;
use serenity::all::CreateAttachment;
use tracing::info;

/// Generate an ICS file for a course
#[poise::command(slash_command, prefix_command)]
pub async fn ics(
    ctx: Context<'_>,
    #[description = "Course Reference Number (CRN)"] crn: i32,
) -> Result<(), Error> {
    ctx.defer().await?;

    let course = utils::get_course_by_crn(&ctx, crn).await?;
    let term = course.term.clone();

    // Get meeting times
    let meeting_times = ctx
        .data()
        .app_state
        .banner_api
        .get_course_meeting_time(&term, &crn.to_string())
        .await?;

    if meeting_times.is_empty() {
        ctx.say("No meeting times found for this course.").await?;
        return Ok(());
    }

    // Sort meeting times by start time
    let mut sorted_meeting_times = meeting_times.to_vec();
    sorted_meeting_times.sort_unstable_by(|a, b| match (&a.time_range, &b.time_range) {
        (Some(a_time), Some(b_time)) => a_time.start.cmp(&b_time.start),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.days.bits().cmp(&b.days.bits()),
    });

    // Generate ICS content
    let ics_content = generate_ics_content(&course, &term, &sorted_meeting_times)?;

    // Create file attachment
    let filename = format!(
        "{subject}_{number}_{section}.ics",
        subject = course.subject.replace(" ", "_"),
        number = course.course_number,
        section = course.sequence_number,
    );

    let file = CreateAttachment::bytes(ics_content.into_bytes(), filename.clone());

    ctx.send(
        poise::CreateReply::default()
            .content(format!(
                "📅 Generated ICS calendar for **{}**\n\n**Meeting Times:**\n{}",
                course.display_title(),
                sorted_meeting_times
                    .iter()
                    .enumerate()
                    .map(|(i, m)| {
                        let time_info = match &m.time_range {
                            Some(range) => format!(
                                "{} {}",
                                m.days_string().unwrap_or("TBA".to_string()),
                                range.format_12hr()
                            ),
                            None => m.days_string().unwrap_or("TBA".to_string()),
                        };
                        format!("{}. {}", i + 1, time_info)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
            .attachment(file),
    )
    .await?;

    info!(crn = %crn, "ics command completed");
    Ok(())
}

/// Generate ICS content for a course and its meeting times
fn generate_ics_content(
    course: &Course,
    term: &str,
    meeting_times: &[MeetingScheduleInfo],
) -> Result<String, anyhow::Error> {
    let mut ics_content = String::new();

    // ICS header
    ics_content.push_str("BEGIN:VCALENDAR\r\n");
    ics_content.push_str("VERSION:2.0\r\n");
    ics_content.push_str("PRODID:-//Banner Bot//Course Calendar//EN\r\n");
    ics_content.push_str("CALSCALE:GREGORIAN\r\n");
    ics_content.push_str("METHOD:PUBLISH\r\n");

    // Calendar name
    ics_content.push_str(&format!(
        "X-WR-CALNAME:{} - {}\r\n",
        course.display_title(),
        term
    ));

    // Generate events for each meeting time
    for (index, meeting_time) in meeting_times.iter().enumerate() {
        let event_content = generate_event_content(course, meeting_time, index)?;
        ics_content.push_str(&event_content);
    }

    // ICS footer
    ics_content.push_str("END:VCALENDAR\r\n");

    Ok(ics_content)
}

/// Generate ICS event content for a single meeting time
fn generate_event_content(
    course: &Course,
    meeting_time: &MeetingScheduleInfo,
    index: usize,
) -> Result<String, anyhow::Error> {
    let course_title = course.display_title();
    let instructor_name = course.primary_instructor_name();
    let location = meeting_time.place_string();

    // Create event title with meeting index if multiple meetings
    let event_title = if index > 0 {
        format!("{} (Meeting {})", course_title, index + 1)
    } else {
        course_title
    };

    // Create event description
    let description = format!(
        "CRN: {}\\nInstructor: {}\\nDays: {}\\nMeeting Type: {}",
        course.course_reference_number,
        instructor_name,
        meeting_time.days_string().unwrap_or("TBA".to_string()),
        meeting_time.meeting_type.description()
    );

    // Get start and end times
    let (start_dt, end_dt) = meeting_time.datetime_range();

    // Format datetimes for ICS (UTC format)
    let start_utc = start_dt.with_timezone(&Utc);
    let end_utc = end_dt.with_timezone(&Utc);

    let start_str = start_utc.format("%Y%m%dT%H%M%SZ").to_string();
    let end_str = end_utc.format("%Y%m%dT%H%M%SZ").to_string();

    // Generate unique ID for the event
    let uid = format!(
        "{}-{}-{}@banner-bot.local",
        course.course_reference_number,
        index,
        start_utc.timestamp()
    );

    let mut event_content = String::new();

    // Event header
    event_content.push_str("BEGIN:VEVENT\r\n");
    event_content.push_str(&format!("UID:{}\r\n", uid));
    event_content.push_str(&format!("DTSTART:{}\r\n", start_str));
    event_content.push_str(&format!("DTEND:{}\r\n", end_str));
    event_content.push_str(&format!("SUMMARY:{}\r\n", escape_ics_text(&event_title)));
    event_content.push_str(&format!(
        "DESCRIPTION:{}\r\n",
        escape_ics_text(&description)
    ));
    event_content.push_str(&format!("LOCATION:{}\r\n", escape_ics_text(&location)));

    // Add recurrence rule if there are specific days and times
    if !meeting_time.days.is_empty() && meeting_time.time_range.is_some() {
        let days_of_week = meeting_time.days_of_week();
        let by_day: Vec<String> = days_of_week
            .iter()
            .map(|day| day.to_short_string().to_uppercase())
            .collect();

        if !by_day.is_empty() {
            let until_date = meeting_time
                .date_range
                .end
                .format("%Y%m%dT000000Z")
                .to_string();

            event_content.push_str(&format!(
                "RRULE:FREQ=WEEKLY;BYDAY={};UNTIL={}\r\n",
                by_day.join(","),
                until_date
            ));
        }
    }

    // Event footer
    event_content.push_str("END:VEVENT\r\n");

    Ok(event_content)
}

/// Escape text for ICS format
fn escape_ics_text(text: &str) -> String {
    text.replace("\\", "\\\\")
        .replace(";", "\\;")
        .replace(",", "\\,")
        .replace("\n", "\\n")
        .replace("\r", "")
}
