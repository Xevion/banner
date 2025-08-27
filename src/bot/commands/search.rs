//! Course search command implementation.

use crate::banner::{SearchQuery, Term};
use crate::bot::{Context, Error};
use anyhow::anyhow;
use regex::Regex;
use tracing::info;

/// Search for courses with various filters
#[poise::command(slash_command, prefix_command)]
pub async fn search(
    ctx: Context<'_>,
    #[description = "Course title (exact, use autocomplete)"] title: Option<String>,
    #[description = "Course code (e.g. 3743, 3000-3999, 3xxx, 3000-)"] code: Option<String>,
    #[description = "Maximum number of results"] max: Option<i32>,
    #[description = "Keywords in title or description (space separated)"] keywords: Option<String>,
    // #[description = "Instructor name"] instructor: Option<String>,
    // #[description = "Subject (e.g Computer Science/CS, Mathematics/MAT)"] subject: Option<String>,
) -> Result<(), Error> {
    // Defer the response since this might take a while
    ctx.defer().await?;

    // Build the search query
    let mut query = SearchQuery::new().credits(3, 6);

    if let Some(title) = title {
        query = query.title(title);
    }

    if let Some(code) = code {
        let (low, high) = parse_course_code(&code)?;
        query = query.course_numbers(low, high);
    }

    if let Some(keywords) = keywords {
        let keyword_list: Vec<String> =
            keywords.split_whitespace().map(|s| s.to_string()).collect();
        query = query.keywords(keyword_list);
    }

    if let Some(max_results) = max {
        query = query.max_results(max_results.min(25)); // Cap at 25
    }

    let term = Term::get_current().inner().to_string();
    let search_result = ctx
        .data()
        .app_state
        .banner_api
        .search(&term, &query, "subjectDescription", false)
        .await?;

    let response = if let Some(courses) = search_result.data {
        if courses.is_empty() {
            "No courses found with the specified criteria.".to_string()
        } else {
            courses
                .iter()
                .map(|course| {
                    format!(
                        "**{}**: {} ({})",
                        course.display_title(),
                        course.primary_instructor_name(),
                        course.course_reference_number
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    } else {
        "No courses found with the specified criteria.".to_string()
    };

    ctx.say(response).await?;
    info!("search command completed");
    Ok(())
}

/// Parse course code input (e.g, "3743", "3000-3999", "3xxx", "3000-")
fn parse_course_code(input: &str) -> Result<(i32, i32), Error> {
    let input = input.trim();

    // Handle range format (e.g, "3000-3999")
    if input.contains('-') {
        let re = Regex::new(r"(\d{1,4})-(\d{1,4})?").unwrap();
        if let Some(captures) = re.captures(input) {
            let low: i32 = captures[1].parse()?;
            let high = if captures.get(2).is_some() {
                captures[2].parse()?
            } else {
                9999 // Open-ended range
            };

            if low > high {
                return Err(anyhow!(
                    "Invalid range: low value greater than high value"
                ));
            }

            if low < 1000 || high > 9999 {
                return Err(anyhow!("Course codes must be between 1000 and 9999"));
            }

            return Ok((low, high));
        }
        return Err(anyhow!("Invalid range format"));
    }

    // Handle wildcard format (e.g, "34xx")
    if input.contains('x') {
        if input.len() != 4 {
            return Err(anyhow!("Wildcard format must be exactly 4 characters"));
        }

        let re = Regex::new(r"(\d+)(x+)").unwrap();
        if let Some(captures) = re.captures(input) {
            let prefix: i32 = captures[1].parse()?;
            let x_count = captures[2].len();

            let low = prefix * 10_i32.pow(x_count as u32);
            let high = low + 10_i32.pow(x_count as u32) - 1;

            if low < 1000 || high > 9999 {
                return Err(anyhow!("Course codes must be between 1000 and 9999"));
            }

            return Ok((low, high));
        }
        return Err(anyhow!("Invalid wildcard format"));
    }

    // Handle single course code
    if input.len() == 4 {
        let code: i32 = input.parse()?;
        if !(1000..=9999).contains(&code) {
            return Err(anyhow!("Course codes must be between 1000 and 9999"));
        }
        return Ok((code, code));
    }

    Err(anyhow!("Invalid course code format"))
}
