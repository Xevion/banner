//! ICS command implementation for generating calendar files.

use crate::bot::{Context, Error, utils};
use tracing::info;

/// Generate an ICS file for a course
#[poise::command(slash_command, prefix_command)]
pub async fn ics(
    ctx: Context<'_>,
    #[description = "Course Reference Number (CRN)"] crn: i32,
) -> Result<(), Error> {
    ctx.defer().await?;

    let course = utils::get_course_by_crn(&ctx, crn).await?;

    // TODO: Implement actual ICS file generation
    ctx.say(format!(
        "ICS generation for '{}' is not yet implemented.",
        course.display_title()
    ))
    .await?;

    info!(crn = %crn, "ics command completed");
    Ok(())
}
