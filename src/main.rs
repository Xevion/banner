use clap::Parser;
use figment::value::UncasedStr;
use num_format::{Locale, ToFormattedString};
use serenity::all::{ActivityData, ClientBuilder, GatewayIntents};
use tokio::signal;
use tracing::{debug, error, info, warn};
use tracing_subscriber::fmt::format::JsonFields;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::banner::BannerApi;
use crate::bot::{Data, get_commands};
use crate::config::Config;
use crate::scraper::ScraperService;
use crate::services::manager::ServiceManager;
use crate::services::{ServiceResult, bot::BotService, web::WebService};
use crate::state::AppState;
use crate::web::routes::BannerState;
use figment::{Figment, providers::Env};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;

mod banner;
mod bot;
mod config;
mod data;
mod error;
mod formatter;
mod scraper;
mod services;
mod state;
mod web;

#[cfg(debug_assertions)]
const DEFAULT_TRACING_FORMAT: TracingFormat = TracingFormat::Pretty;
#[cfg(not(debug_assertions))]
const DEFAULT_TRACING_FORMAT: TracingFormat = TracingFormat::Json;

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
struct Args {
    /// Log formatter to use
    #[arg(long, value_enum, default_value_t = DEFAULT_TRACING_FORMAT)]
    tracing: TracingFormat,

    /// Services to run (comma-separated). Default: all services
    ///
    /// Examples:
    ///   --services bot,web    # Run only bot and web services
    ///   --services scraper    # Run only the scraper service
    #[arg(long, value_delimiter = ',', conflicts_with = "disable_services")]
    services: Option<Vec<ServiceName>>,

    /// Services to disable (comma-separated)
    ///
    /// Examples:
    ///   --disable-services bot        # Run web and scraper only
    ///   --disable-services bot,web    # Run only the scraper service
    #[arg(long, value_delimiter = ',', conflicts_with = "services")]
    disable_services: Option<Vec<ServiceName>>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum TracingFormat {
    /// Use pretty formatter (default in debug mode)
    Pretty,
    /// Use JSON formatter (default in release mode)
    Json,
}

#[derive(clap::ValueEnum, Clone, Debug, PartialEq)]
enum ServiceName {
    /// Discord bot for course monitoring commands
    Bot,
    /// HTTP server for web interface and API
    Web,
    /// Background service for scraping course data
    Scraper,
}

impl ServiceName {
    /// Get all available services
    fn all() -> Vec<ServiceName> {
        vec![ServiceName::Bot, ServiceName::Web, ServiceName::Scraper]
    }

    /// Convert to string for service registration
    fn as_str(&self) -> &'static str {
        match self {
            ServiceName::Bot => "bot",
            ServiceName::Web => "web",
            ServiceName::Scraper => "scraper",
        }
    }
}

/// Determine which services should be enabled based on CLI arguments
fn determine_enabled_services(args: &Args) -> Result<Vec<ServiceName>, anyhow::Error> {
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

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Parse CLI arguments
    let args = Args::parse();

    // Determine which services should be enabled
    let enabled_services: Vec<ServiceName> =
        determine_enabled_services(&args).expect("Failed to determine enabled services");

    info!(
        enabled_services = ?enabled_services,
        "services configuration loaded"
    );

    // Load configuration first to get log level
    let config: Config = Figment::new()
        .merge(Env::raw().map(|k| {
            if k == UncasedStr::new("RAILWAY_DEPLOYMENT_DRAINING_SECONDS") {
                "SHUTDOWN_TIMEOUT".into()
            } else {
                k.into()
            }
        }))
        .extract()
        .expect("Failed to load config");

    // Configure logging based on config
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        let base_level = &config.log_level;
        EnvFilter::new(format!(
            "warn,banner={},banner::rate_limiter=warn,banner::session=warn,banner::rate_limit_middleware=warn",
            base_level
        ))
    });

    // Select formatter based on CLI args
    let use_pretty = match args.tracing {
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

    // Log application startup context
    info!(
        version = env!("CARGO_PKG_VERSION"),
        environment = if cfg!(debug_assertions) {
            "development"
        } else {
            "production"
        },
        "starting banner"
    );

    // Create database connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Failed to create database pool");

    info!(
        port = config.port,
        shutdown_timeout = format!("{:.2?}", config.shutdown_timeout),
        banner_base_url = config.banner_base_url,
        "configuration loaded"
    );

    // Create BannerApi and AppState
    let banner_api = BannerApi::new_with_config(
        config.banner_base_url.clone(),
        config.rate_limiting.clone().into(),
    )
    .expect("Failed to create BannerApi");

    let banner_api_arc = Arc::new(banner_api);
    let app_state = AppState::new(banner_api_arc.clone(), db_pool.clone());

    // Create BannerState for web service
    let banner_state = BannerState {
        api: banner_api_arc.clone(),
    };

    // Configure the client with your Discord bot token in the environment
    let intents = GatewayIntents::non_privileged();

    let bot_target_guild = config.bot_target_guild;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: get_commands(),
            pre_command: |ctx| {
                Box::pin(async move {
                    let content = match ctx {
                        poise::Context::Application(_) => ctx.invocation_string(),
                        poise::Context::Prefix(prefix) => prefix.msg.content.to_string(),
                    };
                    let channel_name = ctx
                        .channel_id()
                        .name(ctx.http())
                        .await
                        .unwrap_or("unknown".to_string());

                    let span = tracing::Span::current();
                    span.record("command_name", ctx.command().qualified_name.as_str());
                    span.record("invocation", ctx.invocation_string());
                    span.record("msg.content", content.as_str());
                    span.record("msg.author", ctx.author().tag().as_str());
                    span.record("msg.id", ctx.id());
                    span.record("msg.channel_id", ctx.channel_id().get());
                    span.record("msg.channel", channel_name.as_str());

                    tracing::info!(
                        command_name = ctx.command().qualified_name.as_str(),
                        invocation = ctx.invocation_string(),
                        msg.content = %content,
                        msg.author = %ctx.author().tag(),
                        msg.author_id = %ctx.author().id,
                        msg.id = %ctx.id(),
                        msg.channel = %channel_name.as_str(),
                        msg.channel_id = %ctx.channel_id(),
                        "{} invoked by {}",
                        ctx.command().name,
                        ctx.author().tag()
                    );
                })
            },
            on_error: |error| {
                Box::pin(async move {
                    if let Err(e) = poise::builtins::on_error(error).await {
                        tracing::error!(error = %e, "Fatal error while sending error message");
                    }
                    // error!(error = ?error, "command error");
                })
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| {
            let app_state = app_state.clone();
            Box::pin(async move {
                poise::builtins::register_in_guild(
                    ctx,
                    &framework.options().commands,
                    bot_target_guild.into(),
                )
                .await?;
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                // Start status update task
                let status_app_state = app_state.clone();
                let status_ctx = ctx.clone();
                tokio::spawn(async move {
                    let max_interval = Duration::from_secs(300); // 5 minutes
                    let base_interval = Duration::from_secs(30);
                    let mut interval = tokio::time::interval(base_interval);
                    let mut previous_course_count: Option<i64> = None;

                    // This runs once immediately on startup, then with adaptive intervals
                    loop {
                        interval.tick().await;

                        // Get the course count, update the activity if it has changed/hasn't been set this session
                        let course_count = status_app_state.get_course_count().await.unwrap();
                        if previous_course_count.is_none()
                            || previous_course_count != Some(course_count)
                        {
                            status_ctx.set_activity(Some(ActivityData::playing(format!(
                                "Querying {:} classes",
                                course_count.to_formatted_string(&Locale::en)
                            ))));
                        }

                        // Increase or reset the interval
                        interval = tokio::time::interval(
                            // Avoid logging the first 'change'
                            if course_count != previous_course_count.unwrap_or(0) {
                                if previous_course_count.is_some() {
                                    debug!(
                                        new_course_count = course_count,
                                        last_interval = interval.period().as_secs(),
                                        "Course count changed, resetting interval"
                                    );
                                }

                                // Record the new course count
                                previous_course_count = Some(course_count);

                                // Reset to base interval
                                base_interval
                            } else {
                                // Increase interval by 10% (up to maximum)
                                let new_interval = interval.period().mul_f32(1.1).min(max_interval);
                                debug!(
                                    current_course_count = course_count,
                                    last_interval = interval.period().as_secs(),
                                    new_interval = new_interval.as_secs(),
                                    "Course count unchanged, increasing interval"
                                );

                                new_interval
                            },
                        );

                        // Reset the interval, otherwise it will tick again immediately
                        interval.reset();
                    }
                });

                Ok(Data { app_state })
            })
        })
        .build();

    let client = ClientBuilder::new(config.bot_token, intents)
        .framework(framework)
        .await
        .expect("Failed to build client");

    // Extract shutdown timeout before moving config
    let shutdown_timeout = config.shutdown_timeout;
    let port = config.port;

    // Create service manager
    let mut service_manager = ServiceManager::new();

    // Register enabled services with the manager
    if enabled_services.contains(&ServiceName::Bot) {
        let bot_service = Box::new(BotService::new(client));
        service_manager.register_service(ServiceName::Bot.as_str(), bot_service);
    }

    if enabled_services.contains(&ServiceName::Web) {
        let web_service = Box::new(WebService::new(port, banner_state));
        service_manager.register_service(ServiceName::Web.as_str(), web_service);
    }

    if enabled_services.contains(&ServiceName::Scraper) {
        let scraper_service =
            Box::new(ScraperService::new(db_pool.clone(), banner_api_arc.clone()));
        service_manager.register_service(ServiceName::Scraper.as_str(), scraper_service);
    }

    // Check if any services are enabled
    if !service_manager.has_services() {
        error!("No services enabled. Cannot start application.");
        std::process::exit(1);
    }

    // Spawn all registered services
    service_manager.spawn_all();

    // Set up signal handling for both SIGINT (Ctrl+C) and SIGTERM
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("received ctrl+c, gracefully shutting down...");
    };

    #[cfg(unix)]
    let sigterm = async {
        use tokio::signal::unix::{SignalKind, signal};
        let mut sigterm_stream =
            signal(SignalKind::terminate()).expect("Failed to install SIGTERM signal handler");
        sigterm_stream.recv().await;
        info!("received SIGTERM, gracefully shutting down...");
    };

    #[cfg(not(unix))]
    let sigterm = async {
        // On non-Unix systems, create a future that never completes
        // This ensures the select! macro works correctly
        std::future::pending::<()>().await;
    };

    // Main application loop - wait for services or signals
    let mut exit_code = 0;

    tokio::select! {
        (service_name, result) = service_manager.run() => {
            // A service completed unexpectedly
            match result {
                ServiceResult::GracefulShutdown => {
                    info!(service = service_name, "service completed gracefully");
                }
                ServiceResult::NormalCompletion => {
                    warn!(service = service_name, "service completed unexpectedly");
                    exit_code = 1;
                }
                ServiceResult::Error(e) => {
                    error!(service = service_name, error = ?e, "service failed");
                    exit_code = 1;
                }
            }

            // Shutdown remaining services
            match service_manager.shutdown(shutdown_timeout).await {
                Ok(elapsed) => {
                    info!(
                        remaining = format!("{:.2?}", shutdown_timeout - elapsed),
                        "graceful shutdown complete"
                    );
                }
                Err(pending_services) => {
                    warn!(
                        pending_count = pending_services.len(),
                        pending_services = ?pending_services,
                        "graceful shutdown elapsed - {} service(s) did not complete",
                        pending_services.len()
                    );

                    // Non-zero exit code, default to 2 if not set
                    exit_code = if exit_code == 0 { 2 } else { exit_code };
                }
            }
        }
        _ = ctrl_c => {
            // User requested shutdown via Ctrl+C
            info!("user requested shutdown via ctrl+c");
            match service_manager.shutdown(shutdown_timeout).await {
                Ok(elapsed) => {
                    info!(
                        remaining = format!("{:.2?}", shutdown_timeout - elapsed),
                        "graceful shutdown complete"
                    );
                    info!("graceful shutdown complete");
                }
                Err(pending_services) => {
                    warn!(
                        pending_count = pending_services.len(),
                        pending_services = ?pending_services,
                        "graceful shutdown elapsed - {} service(s) did not complete",
                        pending_services.len()
                    );
                    exit_code = 2;
                }
            }
        }
        _ = sigterm => {
            // System requested shutdown via SIGTERM
            info!("system requested shutdown via SIGTERM");
            match service_manager.shutdown(shutdown_timeout).await {
                Ok(elapsed) => {
                    info!(
                        remaining = format!("{:.2?}", shutdown_timeout - elapsed),
                        "graceful shutdown complete"
                    );
                    info!("graceful shutdown complete");
                }
                Err(pending_services) => {
                    warn!(
                        pending_count = pending_services.len(),
                        pending_services = ?pending_services,
                        "graceful shutdown elapsed - {} service(s) did not complete",
                        pending_services.len()
                    );
                    exit_code = 2;
                }
            }
        }
    }

    info!(exit_code, "application shutdown complete");
    std::process::exit(exit_code);
}
