use crate::app_state::AppState;
use crate::error::Error;

pub mod commands;
pub mod utils;

#[derive(Debug)]
pub struct Data {
    pub app_state: AppState,
} // User data, which is stored and accessible in all command invocations
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Get all available commands
pub fn get_commands() -> Vec<poise::Command<Data, Error>> {
    vec![
        commands::search(),
        commands::terms(),
        commands::time(),
        commands::ics(),
        commands::gcal(),
    ]
}
