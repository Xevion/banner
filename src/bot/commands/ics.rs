//! ICS command implementation for generating calendar files.

use crate::bot::{Context, Error};

/// Generate an ICS file for a course
#[poise::command(slash_command, prefix_command)]
pub async fn ics(
    ctx: Context<'_>,
    #[description = "Course Reference Number (CRN)"] crn: i32,
) -> Result<(), Error> {
    ctx.defer().await?;

    // TODO: Get BannerApi from context or global state
    // TODO: Get current term dynamically
    let term = 202510; // Hardcoded for now

    // TODO: Implement actual ICS file generation
    ctx.say(format!(
        "ICS command not yet implemented - BannerApi integration needed\nCRN: {}, Term: {}",
        crn, term
    ))
    .await?;

    Ok(())
}
