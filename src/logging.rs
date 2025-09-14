use crate::cli::TracingFormat;
use crate::config::Config;
use crate::formatter;
use tracing_subscriber::fmt::format::JsonFields;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

/// Configure and initialize logging for the application
pub fn setup_logging(config: &Config, tracing_format: TracingFormat) {
    // Configure logging based on config
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let base_level = &config.log_level;
        EnvFilter::new(format!(
            "warn,banner={},banner::rate_limiter=warn,banner::session=warn,banner::rate_limit_middleware=warn",
            base_level
        ))
    });

    // Select formatter based on CLI args
    let use_pretty = match tracing_format {
        TracingFormat::Pretty => true,
        TracingFormat::Json => false,
    };

    let subscriber: Box<dyn tracing::Subscriber + Send + Sync> = if use_pretty {
        Box::new(
            FmtSubscriber::builder()
                .with_target(true)
                .event_format(formatter::CustomPrettyFormatter)
                .with_env_filter(filter)
                .finish(),
        )
    } else {
        Box::new(
            FmtSubscriber::builder()
                .with_target(true)
                .event_format(formatter::CustomJsonFormatter)
                .fmt_fields(JsonFields::new())
                .with_env_filter(filter)
                .finish(),
        )
    };

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}
