//! Terms command implementation.

use crate::bot::{Context, Error};

/// List available terms or search for a specific term
#[poise::command(slash_command, prefix_command)]
pub async fn terms(
    ctx: Context<'_>,
    #[description = "Term to search for"] search: Option<String>,
    #[description = "Page number"] page: Option<i32>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let search_term = search.unwrap_or_default();
    let page_number = page.unwrap_or(1).max(1);

    // TODO: Get BannerApi from context or global state
    // For now, we'll return a placeholder response
    ctx.say(format!(
        "Terms command not yet implemented - BannerApi integration needed\nSearch: '{}', Page: {}",
        search_term, page_number
    ))
    .await?;

    Ok(())
}
