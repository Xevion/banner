//! Time command implementation for course meeting times.

use crate::bot::{utils, Context, Error};
use tracing::info;

/// Get meeting times for a specific course
#[poise::command(slash_command, prefix_command)]
pub async fn time(
    ctx: Context<'_>,
    #[description = "Course Reference Number (CRN)"] crn: i32,
) -> Result<(), Error> {
    ctx.defer().await?;

    let course = utils::get_course_by_crn(&ctx, crn).await?;

    // TODO: Implement actual meeting time retrieval and display
    ctx.say(format!(
        "Meeting time display for '{}' is not yet implemented.",
        course.display_title()
    ))
    .await?;

    info!("time command completed for CRN: {}", crn);
    Ok(())
}
