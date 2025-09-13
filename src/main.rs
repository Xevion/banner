use clap::Parser;
use figment::value::UncasedStr;
use num_format::{Locale, ToFormattedString};
use serenity::all::{ActivityData, ClientBuilder, Context, GatewayIntents};
use tokio::signal;
use tracing::{error, info, warn};
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

/// Banner Discord Bot - Course availability monitoring
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Log formatter to use
    #[arg(long, value_enum, default_value_t = LogFormatter::Auto)]
    formatter: LogFormatter,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum LogFormatter {
    /// Use pretty formatter (default in debug mode)
    Pretty,
    /// Use JSON formatter (default in release mode)
    Json,
    /// Auto-select based on build mode (debug=pretty, release=json)
    Auto,
}

async fn update_bot_status(ctx: &Context, app_state: &AppState) -> Result<(), anyhow::Error> {
    let course_count = app_state.get_course_count().await?;

    ctx.set_activity(Some(ActivityData::playing(format!(
        "Querying {:} classes",
        course_count.to_formatted_string(&Locale::en)
    ))));

    tracing::info!(course_count = course_count, "Updated bot status");
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Parse CLI arguments
    let args = Args::parse();

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
        EnvFilter::new(&format!(
            "warn,banner={},banner::rate_limiter=warn,banner::session=warn,banner::rate_limit_middleware=warn",
            base_level
        ))
    });

    // Select formatter based on CLI args
    let use_pretty = match args.formatter {
        LogFormatter::Pretty => true,
        LogFormatter::Json => false,
        LogFormatter::Auto => cfg!(debug_assertions),
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
                    span.record("msg.channel", &channel_name.as_str());

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
                    let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));

                    // Update status immediately on startup
                    if let Err(e) = update_bot_status(&status_ctx, &status_app_state).await {
                        tracing::error!(error = %e, "Failed to update status on startup");
                    }

                    loop {
                        interval.tick().await;

                        if let Err(e) = update_bot_status(&status_ctx, &status_app_state).await {
                            tracing::error!(error = %e, "Failed to update bot status");
                        }
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

    // Register services with the manager
    let bot_service = Box::new(BotService::new(client));
    let web_service = Box::new(WebService::new(port, banner_state));
    let scraper_service = Box::new(ScraperService::new(db_pool.clone(), banner_api_arc.clone()));

    service_manager.register_service("bot", bot_service);
    service_manager.register_service("web", web_service);
    service_manager.register_service("scraper", scraper_service);

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
