use clap::Parser;

/// Banner Discord Bot - Course availability monitoring
///
/// This application runs multiple services that can be controlled via CLI arguments:
/// - bot: Discord bot for course monitoring commands
/// - web: HTTP server for web interface and API
/// - scraper: Background service for scraping course data
///
/// Use --services to specify which services to run, or --disable-services to exclude specific services.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Log formatter to use
    #[arg(long, value_enum, default_value_t = default_tracing_format())]
    pub tracing: TracingFormat,

    /// Services to run (comma-separated). Default: all services
    ///
    /// Examples:
    ///   --services bot,web    # Run only bot and web services
    ///   --services scraper    # Run only the scraper service
    #[arg(long, value_delimiter = ',', conflicts_with = "disable_services")]
    pub services: Option<Vec<ServiceName>>,

    /// Services to disable (comma-separated)
    ///
    /// Examples:
    ///   --disable-services bot        # Run web and scraper only
    ///   --disable-services bot,web    # Run only the scraper service
    #[arg(long, value_delimiter = ',', conflicts_with = "services")]
    pub disable_services: Option<Vec<ServiceName>>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum TracingFormat {
    /// Use pretty formatter (default in debug mode)
    Pretty,
    /// Use JSON formatter (default in release mode)
    Json,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
pub enum ServiceName {
    /// Discord bot for course monitoring commands
    Bot,
    /// HTTP server for web interface and API
    Web,
    /// Background service for scraping course data
    Scraper,
}

impl ServiceName {
    /// Get all available services
    pub fn all() -> Vec<ServiceName> {
        vec![ServiceName::Bot, ServiceName::Web, ServiceName::Scraper]
    }

    /// Convert to string for service registration
    pub fn as_str(&self) -> &'static str {
        match self {
            ServiceName::Bot => "bot",
            ServiceName::Web => "web",
            ServiceName::Scraper => "scraper",
        }
    }
}

/// Determine which services should be enabled based on CLI arguments
pub fn determine_enabled_services(args: &Args) -> Result<Vec<ServiceName>, anyhow::Error> {
    match (&args.services, &args.disable_services) {
        (Some(services), None) => {
            // User specified which services to run
            Ok(services.clone())
        }
        (None, Some(disabled)) => {
            // User specified which services to disable
            let enabled: Vec<ServiceName> = ServiceName::all()
                .into_iter()
                .filter(|s| !disabled.contains(s))
                .collect();
            Ok(enabled)
        }
        (None, None) => {
            // Default: run all services
            Ok(ServiceName::all())
        }
        (Some(_), Some(_)) => {
            // This should be prevented by clap's conflicts_with, but just in case
            Err(anyhow::anyhow!(
                "Cannot specify both --services and --disable-services"
            ))
        }
    }
}

#[cfg(debug_assertions)]
const DEFAULT_TRACING_FORMAT: TracingFormat = TracingFormat::Pretty;
#[cfg(not(debug_assertions))]
const DEFAULT_TRACING_FORMAT: TracingFormat = TracingFormat::Json;

fn default_tracing_format() -> TracingFormat {
    DEFAULT_TRACING_FORMAT
}
