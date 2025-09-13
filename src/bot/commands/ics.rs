//! ICS command implementation for generating calendar files.

use crate::banner::{Course, MeetingScheduleInfo};
use crate::bot::{Context, Error, utils};
use chrono::{Datelike, NaiveDate, Utc};
use serenity::all::CreateAttachment;
use tracing::info;

/// Represents a holiday or special day that should be excluded from class schedules
#[derive(Debug, Clone)]
enum Holiday {
    /// A single-day holiday
    Single { month: u32, day: u32 },
    /// A multi-day holiday range
    Range {
        month: u32,
        start_day: u32,
        end_day: u32,
    },
}

impl Holiday {
    /// Check if a specific date falls within this holiday
    fn contains_date(&self, date: NaiveDate) -> bool {
        match self {
            Holiday::Single { month, day, .. } => date.month() == *month && date.day() == *day,
            Holiday::Range {
                month,
                start_day,
                end_day,
                ..
            } => date.month() == *month && date.day() >= *start_day && date.day() <= *end_day,
        }
    }

    /// Get all dates in this holiday for a given year
    fn get_dates_for_year(&self, year: i32) -> Vec<NaiveDate> {
        match self {
            Holiday::Single { month, day, .. } => {
                if let Some(date) = NaiveDate::from_ymd_opt(year, *month, *day) {
                    vec![date]
                } else {
                    Vec::new()
                }
            }
            Holiday::Range {
                month,
                start_day,
                end_day,
                ..
            } => {
                let mut dates = Vec::new();
                for day in *start_day..=*end_day {
                    if let Some(date) = NaiveDate::from_ymd_opt(year, *month, day) {
                        dates.push(date);
                    }
                }
                dates
            }
        }
    }
}

/// University holidays that should be excluded from class schedules
const UNIVERSITY_HOLIDAYS: &[(&str, Holiday)] = &[
    ("Labor Day", Holiday::Single { month: 9, day: 1 }),
    (
        "Fall Break",
        Holiday::Range {
            month: 10,
            start_day: 13,
            end_day: 14,
        },
    ),
    (
        "Unspecified Holiday",
        Holiday::Single { month: 11, day: 26 },
    ),
    (
        "Thanksgiving",
        Holiday::Range {
            month: 11,
            start_day: 28,
            end_day: 29,
        },
    ),
    ("Student Study Day", Holiday::Single { month: 12, day: 5 }),
    (
        "Winter Holiday",
        Holiday::Range {
            month: 12,
            start_day: 23,
            end_day: 31,
        },
    ),
    ("New Year's Day", Holiday::Single { month: 1, day: 1 }),
    ("MLK Day", Holiday::Single { month: 1, day: 20 }),
    (
        "Spring Break",
        Holiday::Range {
            month: 3,
            start_day: 10,
            end_day: 15,
        },
    ),
    ("Student Study Day", Holiday::Single { month: 5, day: 9 }),
];

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
    let (ics_content, excluded_holidays) =
        generate_ics_content(&course, &term, &sorted_meeting_times)?;

    // Create file attachment
    let filename = format!(
        "{subject}_{number}_{section}.ics",
        subject = course.subject.replace(" ", "_"),
        number = course.course_number,
        section = course.sequence_number,
    );

    let file = CreateAttachment::bytes(ics_content.into_bytes(), filename.clone());

    // Build response content
    let mut response_content = format!(
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
    );

    // Add holiday exclusion information
    if !excluded_holidays.is_empty() {
        let count = excluded_holidays.len();
        let count_text = if count == 1 {
            "1 date was".to_string()
        } else {
            format!("{} dates were", count)
        };
        response_content.push_str(&format!("\n\n{} excluded from the ICS file:\n", count_text));
        response_content.push_str(
            &excluded_holidays
                .iter()
                .map(|s| format!("- {}", s))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }

    ctx.send(
        poise::CreateReply::default()
            .content(response_content)
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
) -> Result<(String, Vec<String>), anyhow::Error> {
    let mut ics_content = String::new();
    let mut excluded_holidays = Vec::new();

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
        let (event_content, holidays) = generate_event_content(course, meeting_time, index)?;
        ics_content.push_str(&event_content);
        excluded_holidays.extend(holidays);
    }

    // ICS footer
    ics_content.push_str("END:VCALENDAR\r\n");

    Ok((ics_content, excluded_holidays))
}

/// Generate ICS event content for a single meeting time
fn generate_event_content(
    course: &Course,
    meeting_time: &MeetingScheduleInfo,
    index: usize,
) -> Result<(String, Vec<String>), anyhow::Error> {
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

            // Add holiday exceptions (EXDATE) if the class would meet on holiday dates
            let holiday_exceptions = get_holiday_exceptions(meeting_time);
            if let Some(exdate_property) = generate_exdate_property(&holiday_exceptions, start_utc)
            {
                event_content.push_str(&format!("{}\r\n", exdate_property));
            }

            // Collect holiday names for reporting
            let mut holiday_names = Vec::new();
            for (holiday_name, holiday) in UNIVERSITY_HOLIDAYS {
                for &exception_date in &holiday_exceptions {
                    if holiday.contains_date(exception_date) {
                        holiday_names.push(format!(
                            "{} ({})",
                            holiday_name,
                            exception_date.format("%a, %b %d")
                        ));
                    }
                }
            }
            holiday_names.sort();
            holiday_names.dedup();

            return Ok((event_content, holiday_names));
        }
    }

    // Event footer
    event_content.push_str("END:VEVENT\r\n");

    Ok((event_content, Vec::new()))
}

/// Convert chrono::Weekday to the custom DayOfWeek enum
fn chrono_weekday_to_day_of_week(weekday: chrono::Weekday) -> crate::banner::meetings::DayOfWeek {
    use crate::banner::meetings::DayOfWeek;
    match weekday {
        chrono::Weekday::Mon => DayOfWeek::Monday,
        chrono::Weekday::Tue => DayOfWeek::Tuesday,
        chrono::Weekday::Wed => DayOfWeek::Wednesday,
        chrono::Weekday::Thu => DayOfWeek::Thursday,
        chrono::Weekday::Fri => DayOfWeek::Friday,
        chrono::Weekday::Sat => DayOfWeek::Saturday,
        chrono::Weekday::Sun => DayOfWeek::Sunday,
    }
}

/// Check if a class meets on a specific date based on its meeting days
fn class_meets_on_date(meeting_time: &MeetingScheduleInfo, date: NaiveDate) -> bool {
    let weekday = chrono_weekday_to_day_of_week(date.weekday());
    let meeting_days = meeting_time.days_of_week();

    meeting_days.contains(&weekday)
}

/// Get holiday dates that fall within the course date range and would conflict with class meetings
fn get_holiday_exceptions(meeting_time: &MeetingScheduleInfo) -> Vec<NaiveDate> {
    let mut exceptions = Vec::new();

    // Get the year range from the course date range
    let start_year = meeting_time.date_range.start.year();
    let end_year = meeting_time.date_range.end.year();

    for (_, holiday) in UNIVERSITY_HOLIDAYS {
        // Check for the holiday in each year of the course
        for year in start_year..=end_year {
            let holiday_dates = holiday.get_dates_for_year(year);

            for holiday_date in holiday_dates {
                // Check if the holiday falls within the course date range
                if holiday_date >= meeting_time.date_range.start
                    && holiday_date <= meeting_time.date_range.end
                {
                    // Check if the class would actually meet on this day
                    if class_meets_on_date(meeting_time, holiday_date) {
                        exceptions.push(holiday_date);
                    }
                }
            }
        }
    }

    exceptions
}

/// Generate EXDATE property for holiday exceptions
fn generate_exdate_property(
    exceptions: &[NaiveDate],
    start_time: chrono::DateTime<Utc>,
) -> Option<String> {
    if exceptions.is_empty() {
        return None;
    }

    let mut exdate_values = Vec::new();

    for &exception_date in exceptions {
        // Create a datetime for the exception using the same time as the start time
        let exception_datetime = exception_date.and_time(start_time.time()).and_utc();

        let exdate_str = exception_datetime.format("%Y%m%dT%H%M%SZ").to_string();
        exdate_values.push(exdate_str);
    }

    Some(format!("EXDATE:{}", exdate_values.join(",")))
}

/// Escape text for ICS format
fn escape_ics_text(text: &str) -> String {
    text.replace("\\", "\\\\")
        .replace(";", "\\;")
        .replace(",", "\\,")
        .replace("\n", "\\n")
        .replace("\r", "")
}
