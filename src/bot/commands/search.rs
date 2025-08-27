//! Course search command implementation.

use crate::banner::SearchQuery;
use crate::bot::{Context, Error};
use regex::Regex;

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

    // TODO: Get current term dynamically
    // TODO: Get BannerApi from context or global state
    // For now, we'll return an error
    ctx.say("Search functionality not yet implemented - BannerApi integration needed")
        .await?;

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
                return Err("Invalid range: low value greater than high value".into());
            }

            if low < 1000 || high > 9999 {
                return Err("Course codes must be between 1000 and 9999".into());
            }

            return Ok((low, high));
        }
        return Err("Invalid range format".into());
    }

    // Handle wildcard format (e.g, "34xx")
    if input.contains('x') {
        if input.len() != 4 {
            return Err("Wildcard format must be exactly 4 characters".into());
        }

        let re = Regex::new(r"(\d+)(x+)").unwrap();
        if let Some(captures) = re.captures(input) {
            let prefix: i32 = captures[1].parse()?;
            let x_count = captures[2].len();

            let low = prefix * 10_i32.pow(x_count as u32);
            let high = low + 10_i32.pow(x_count as u32) - 1;

            if low < 1000 || high > 9999 {
                return Err("Course codes must be between 1000 and 9999".into());
            }

            return Ok((low, high));
        }
        return Err("Invalid wildcard format".into());
    }

    // Handle single course code
    if input.len() == 4 {
        let code: i32 = input.parse()?;
        if !(1000..=9999).contains(&code) {
            return Err("Course codes must be between 1000 and 9999".into());
        }
        return Ok((code, code));
    }

    Err("Invalid course code format".into())
}
