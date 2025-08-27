//! Time command implementation for course meeting times.

use crate::bot::{Context, Error};

/// Get meeting times for a specific course
#[poise::command(slash_command, prefix_command)]
pub async fn time(
    ctx: Context<'_>,
    #[description = "Course Reference Number (CRN)"] crn: i32,
) -> Result<(), Error> {
    ctx.defer().await?;

    // TODO: Get BannerApi from context or global state
    // TODO: Get current term dynamically
    let term = 202510; // Hardcoded for now

    // TODO: Implement actual meeting time retrieval
    ctx.say(format!(
        "Time command not yet implemented - BannerApi integration needed\nCRN: {}, Term: {}",
        crn, term
    ))
    .await?;

    Ok(())
}
