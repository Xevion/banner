//! Query builder for Banner API course searches.

use std::collections::HashMap;
use std::time::Duration;

/// Range of two integers
#[derive(Debug, Clone)]
pub struct Range {
    pub low: i32,
    pub high: i32,
}

/// Builder for constructing Banner API search queries
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    subject: Option<String>,
    title: Option<String>,
    keywords: Option<Vec<String>>,
    course_reference_number: Option<String>,
    open_only: Option<bool>,
    term_part: Option<Vec<String>>,
    campus: Option<Vec<String>>,
    instructional_method: Option<Vec<String>>,
    attributes: Option<Vec<String>>,
    instructor: Option<Vec<u64>>,
    start_time: Option<Duration>,
    end_time: Option<Duration>,
    min_credits: Option<i32>,
    max_credits: Option<i32>,
    offset: i32,
    max_results: i32,
    course_number_range: Option<Range>,
}

impl SearchQuery {
    /// Creates a new SearchQuery with default values
    pub fn new() -> Self {
        Self {
            max_results: 8,
            offset: 0,
            ..Default::default()
        }
    }

    /// Sets the subject for the query
    pub fn subject<S: Into<String>>(mut self, subject: S) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Sets the title for the query
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the course reference number (CRN) for the query
    pub fn course_reference_number<S: Into<String>>(mut self, crn: S) -> Self {
        self.course_reference_number = Some(crn.into());
        self
    }

    /// Sets the keywords for the query
    pub fn keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = Some(keywords);
        self
    }

    /// Adds a keyword to the query
    pub fn keyword<S: Into<String>>(mut self, keyword: S) -> Self {
        match &mut self.keywords {
            Some(keywords) => keywords.push(keyword.into()),
            None => self.keywords = Some(vec![keyword.into()]),
        }
        self
    }

    /// Sets whether to search for open courses only
    pub fn open_only(mut self, open_only: bool) -> Self {
        self.open_only = Some(open_only);
        self
    }

    /// Sets the term part for the query
    pub fn term_part(mut self, term_part: Vec<String>) -> Self {
        self.term_part = Some(term_part);
        self
    }

    /// Sets the campuses for the query
    pub fn campus(mut self, campus: Vec<String>) -> Self {
        self.campus = Some(campus);
        self
    }

    /// Sets the instructional methods for the query
    pub fn instructional_method(mut self, instructional_method: Vec<String>) -> Self {
        self.instructional_method = Some(instructional_method);
        self
    }

    /// Sets the attributes for the query
    pub fn attributes(mut self, attributes: Vec<String>) -> Self {
        self.attributes = Some(attributes);
        self
    }

    /// Sets the instructors for the query
    pub fn instructor(mut self, instructor: Vec<u64>) -> Self {
        self.instructor = Some(instructor);
        self
    }

    /// Sets the start time for the query
    pub fn start_time(mut self, start_time: Duration) -> Self {
        self.start_time = Some(start_time);
        self
    }

    /// Sets the end time for the query
    pub fn end_time(mut self, end_time: Duration) -> Self {
        self.end_time = Some(end_time);
        self
    }

    /// Sets the credit range for the query
    pub fn credits(mut self, low: i32, high: i32) -> Self {
        self.min_credits = Some(low);
        self.max_credits = Some(high);
        self
    }

    /// Sets the minimum credits for the query
    pub fn min_credits(mut self, value: i32) -> Self {
        self.min_credits = Some(value);
        self
    }

    /// Sets the maximum credits for the query
    pub fn max_credits(mut self, value: i32) -> Self {
        self.max_credits = Some(value);
        self
    }

    /// Sets the course number range for the query
    pub fn course_numbers(mut self, low: i32, high: i32) -> Self {
        self.course_number_range = Some(Range { low, high });
        self
    }

    /// Sets the offset for pagination
    pub fn offset(mut self, offset: i32) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the maximum number of results to return
    pub fn max_results(mut self, max_results: i32) -> Self {
        self.max_results = max_results;
        self
    }

    /// Converts the query into URL parameters for the Banner API
    pub fn to_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();

        if let Some(ref subject) = self.subject {
            params.insert("txt_subject".to_string(), subject.clone());
        }

        if let Some(ref title) = self.title {
            params.insert("txt_courseTitle".to_string(), title.trim().to_string());
        }

        if let Some(ref crn) = self.course_reference_number {
            params.insert("txt_courseReferenceNumber".to_string(), crn.clone());
        }

        if let Some(ref keywords) = self.keywords {
            params.insert("txt_keywordlike".to_string(), keywords.join(" "));
        }

        if self.open_only.is_some() {
            params.insert("chk_open_only".to_string(), "true".to_string());
        }

        if let Some(ref term_part) = self.term_part {
            params.insert("txt_partOfTerm".to_string(), term_part.join(","));
        }

        if let Some(ref campus) = self.campus {
            params.insert("txt_campus".to_string(), campus.join(","));
        }

        if let Some(ref attributes) = self.attributes {
            params.insert("txt_attribute".to_string(), attributes.join(","));
        }

        if let Some(ref instructor) = self.instructor {
            let instructor_str = instructor
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(",");
            params.insert("txt_instructor".to_string(), instructor_str);
        }

        if let Some(start_time) = self.start_time {
            let (hour, minute, meridiem) = format_time_parameter(start_time);
            params.insert("select_start_hour".to_string(), hour);
            params.insert("select_start_min".to_string(), minute);
            params.insert("select_start_ampm".to_string(), meridiem);
        }

        if let Some(end_time) = self.end_time {
            let (hour, minute, meridiem) = format_time_parameter(end_time);
            params.insert("select_end_hour".to_string(), hour);
            params.insert("select_end_min".to_string(), minute);
            params.insert("select_end_ampm".to_string(), meridiem);
        }

        if let Some(min_credits) = self.min_credits {
            params.insert("txt_credithourlow".to_string(), min_credits.to_string());
        }

        if let Some(max_credits) = self.max_credits {
            params.insert("txt_credithourhigh".to_string(), max_credits.to_string());
        }

        if let Some(ref range) = self.course_number_range {
            params.insert("txt_course_number_range".to_string(), range.low.to_string());
            params.insert(
                "txt_course_number_range_to".to_string(),
                range.high.to_string(),
            );
        }

        params.insert("pageOffset".to_string(), self.offset.to_string());
        params.insert("pageMaxSize".to_string(), self.max_results.to_string());

        params
    }
}

/// Formats a Duration into hour, minute, and meridiem strings for Banner API
fn format_time_parameter(duration: Duration) -> (String, String, String) {
    let total_minutes = duration.as_secs() / 60;
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    let minute_str = minutes.to_string();

    if hours >= 12 {
        let meridiem = "PM".to_string();
        let hour_str = if hours >= 13 {
            (hours - 12).to_string()
        } else {
            hours.to_string()
        };
        (hour_str, minute_str, meridiem)
    } else {
        let meridiem = "AM".to_string();
        let hour_str = hours.to_string();
        (hour_str, minute_str, meridiem)
    }
}

impl std::fmt::Display for SearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();

        if let Some(ref subject) = self.subject {
            parts.push(format!("subject={}", subject));
        }
        if let Some(ref title) = self.title {
            parts.push(format!("title={}", title.trim()));
        }
        if let Some(ref keywords) = self.keywords {
            parts.push(format!("keywords={}", keywords.join(" ")));
        }
        if self.open_only.is_some() {
            parts.push("openOnly=true".to_string());
        }
        if let Some(ref term_part) = self.term_part {
            parts.push(format!("termPart={}", term_part.join(",")));
        }
        if let Some(ref campus) = self.campus {
            parts.push(format!("campus={}", campus.join(",")));
        }
        if let Some(ref attributes) = self.attributes {
            parts.push(format!("attributes={}", attributes.join(",")));
        }
        if let Some(ref instructor) = self.instructor {
            let instructor_str = instructor
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(",");
            parts.push(format!("instructor={}", instructor_str));
        }
        if let Some(start_time) = self.start_time {
            let (hour, minute, meridiem) = format_time_parameter(start_time);
            parts.push(format!("startTime={}:{}:{}", hour, minute, meridiem));
        }
        if let Some(end_time) = self.end_time {
            let (hour, minute, meridiem) = format_time_parameter(end_time);
            parts.push(format!("endTime={}:{}:{}", hour, minute, meridiem));
        }
        if let Some(min_credits) = self.min_credits {
            parts.push(format!("minCredits={}", min_credits));
        }
        if let Some(max_credits) = self.max_credits {
            parts.push(format!("maxCredits={}", max_credits));
        }
        if let Some(ref range) = self.course_number_range {
            parts.push(format!("courseNumberRange={}-{}", range.low, range.high));
        }

        parts.push(format!("offset={}", self.offset));
        parts.push(format!("maxResults={}", self.max_results));

        write!(f, "{}", parts.join(", "))
    }
}
