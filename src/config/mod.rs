//! Configuration module for the banner application.
//!
//! This module handles loading and parsing configuration from environment variables
//! using the figment crate. It supports flexible duration parsing that accepts both
//! numeric values (interpreted as seconds) and duration strings with units.

use fundu::{DurationParser, TimeUnit};
use serde::{Deserialize, Deserializer};
use std::time::Duration;

/// Main application configuration containing all sub-configurations
#[derive(Deserialize)]
pub struct Config {
    /// Log level for the application
    ///
    /// This value is used to set the log level for this application's target specifically.
    /// e.g. "debug" would be similar to "warn,banner=debug,..."
    ///
    /// Valid values are: "trace", "debug", "info", "warn", "error"
    /// Defaults to "info" if not specified
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Port for the web server (default: 8080)
    #[serde(default = "default_port")]
    pub port: u16,
    /// Database connection URL
    pub database_url: String,
    /// Graceful shutdown timeout duration
    ///
    /// Accepts both numeric values (seconds) and duration strings
    /// Defaults to 8 seconds if not specified
    #[serde(
        default = "default_shutdown_timeout",
        deserialize_with = "deserialize_duration"
    )]
    pub shutdown_timeout: Duration,
    /// Discord bot token for authentication
    pub bot_token: String,
    /// Target Discord guild ID where the bot operates
    pub bot_target_guild: u64,

    /// Base URL for banner generation service
    ///
    /// Defaults to "https://ssbprod.utsa.edu/StudentRegistrationSsb/ssb" if not specified
    #[serde(default = "default_banner_base_url")]
    pub banner_base_url: String,
    /// Rate limiting configuration for Banner API requests
    #[serde(default = "default_rate_limiting")]
    pub rate_limiting: RateLimitingConfig,
}

/// Default log level of "info"
fn default_log_level() -> String {
    "info".to_string()
}

/// Default port of 8080
fn default_port() -> u16 {
    8080
}

/// Default shutdown timeout of 8 seconds
fn default_shutdown_timeout() -> Duration {
    Duration::from_secs(8)
}

/// Default banner base URL
fn default_banner_base_url() -> String {
    "https://ssbprod.utsa.edu/StudentRegistrationSsb/ssb".to_string()
}

/// Rate limiting configuration for Banner API requests
#[derive(Deserialize, Clone, Debug)]
pub struct RateLimitingConfig {
    /// Requests per minute for session operations (very conservative)
    #[serde(default = "default_session_rpm")]
    pub session_rpm: u32,
    /// Requests per minute for search operations (moderate)
    #[serde(default = "default_search_rpm")]
    pub search_rpm: u32,
    /// Requests per minute for metadata operations (moderate)
    #[serde(default = "default_metadata_rpm")]
    pub metadata_rpm: u32,
    /// Requests per minute for reset operations (low priority)
    #[serde(default = "default_reset_rpm")]
    pub reset_rpm: u32,
    /// Burst allowance (extra requests allowed in short bursts)
    #[serde(default = "default_burst_allowance")]
    pub burst_allowance: u32,
}

/// Default rate limiting configuration
fn default_rate_limiting() -> RateLimitingConfig {
    RateLimitingConfig {
        session_rpm: default_session_rpm(),
        search_rpm: default_search_rpm(),
        metadata_rpm: default_metadata_rpm(),
        reset_rpm: default_reset_rpm(),
        burst_allowance: default_burst_allowance(),
    }
}

/// Default session requests per minute (6 = 1 every 10 seconds)
fn default_session_rpm() -> u32 {
    6
}

/// Default search requests per minute (30 = 1 every 2 seconds)
fn default_search_rpm() -> u32 {
    30
}

/// Default metadata requests per minute (20 = 1 every 3 seconds)
fn default_metadata_rpm() -> u32 {
    20
}

/// Default reset requests per minute (10 = 1 every 6 seconds)
fn default_reset_rpm() -> u32 {
    10
}

/// Default burst allowance (3 extra requests)
fn default_burst_allowance() -> u32 {
    3
}

/// Duration parser configured to handle various time units with seconds as default
///
/// Supports:
/// - Seconds (s) - default unit
/// - Milliseconds (ms)
/// - Minutes (m)
/// - Hours (h)
///
/// Does not support fractions, exponents, or infinity values
/// Allows for whitespace between the number and the time unit
/// Allows for multiple time units to be specified (summed together, e.g "10s 2m" = 120 + 10 = 130 seconds)
const DURATION_PARSER: DurationParser<'static> = DurationParser::builder()
    .time_units(&[TimeUnit::Second, TimeUnit::MilliSecond, TimeUnit::Minute])
    .parse_multiple(None)
    .allow_time_unit_delimiter()
    .disable_infinity()
    .disable_fraction()
    .disable_exponent()
    .default_unit(TimeUnit::Second)
    .build();

/// Custom deserializer for duration fields that accepts both numeric and string values
///
/// This deserializer handles the flexible duration parsing by accepting:
/// - Unsigned integers (interpreted as seconds)
/// - Signed integers (interpreted as seconds, must be non-negative)
/// - Strings (parsed using the fundu duration parser)
///
/// # Examples
///
/// - `1` -> 1 second
/// - `"30s"` -> 30 seconds  
/// - `"2 m"` -> 2 minutes
/// - `"1500ms"` -> 15 seconds
fn deserialize_duration<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Visitor;

    struct DurationVisitor;

    impl<'de> Visitor<'de> for DurationVisitor {
        type Value = Duration;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a duration string or number")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            DURATION_PARSER.parse(value)
                .map_err(|e| {
                    serde::de::Error::custom(format!(
                        "Invalid duration format '{}': {}. Examples: '5' (5 seconds), '3500ms', '30s', '2m', '1.5h'", 
                        value, e
                    ))
                })?
                .try_into()
                .map_err(|e| serde::de::Error::custom(format!("Duration conversion error: {}", e)))
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Duration::from_secs(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            if value < 0 {
                return Err(serde::de::Error::custom("Duration cannot be negative"));
            }
            Ok(Duration::from_secs(value as u64))
        }
    }

    deserializer.deserialize_any(DurationVisitor)
}
