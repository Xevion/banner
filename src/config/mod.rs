use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub bot_token: String,
    pub database_url: String,
    pub redis_url: String,
    pub banner_base_url: String,
    pub bot_target_guild: u64,
    pub bot_app_id: u64,
}
