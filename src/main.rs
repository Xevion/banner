use serenity::all::{CacheHttp, ClientBuilder, GatewayIntents};
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::app_state::AppState;
use crate::banner::BannerApi;
use crate::bot::{Data, get_commands};
use crate::config::Config;
use crate::services::manager::ServiceManager;
use crate::services::{ServiceResult, bot::BotService, web::WebService};
use crate::web::routes::BannerState;
use figment::{Figment, providers::Env};
use std::sync::Arc;

mod app_state;
mod banner;
mod bot;
mod config;
mod data;
mod error;
mod services;
mod web;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Configure logging
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn,banner=debug"));
    let subscriber = {
        #[cfg(debug_assertions)]
        {
            FmtSubscriber::builder()
        }
        #[cfg(not(debug_assertions))]
        {
            FmtSubscriber::builder().json()
        }
    }
    .with_env_filter(filter)
    .with_target(true)
    .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Log application startup context
    info!(
        version = env!("CARGO_PKG_VERSION"),
        environment = if cfg!(debug_assertions) {
            "development"
        } else {
            "production"
        },
        "starting banner system"
    );

    let config: Config = Figment::new()
        .merge(Env::prefixed("APP_"))
        .extract()
        .expect("Failed to load config");

    info!(
        port = config.port,
        shutdown_timeout = format!("{:.2?}", config.shutdown_timeout),
        banner_base_url = config.banner_base_url,
        "configuration loaded"
    );

    // Create BannerApi and AppState
    let banner_api =
        BannerApi::new(config.banner_base_url.clone()).expect("Failed to create BannerApi");
    banner_api
        .setup()
        .await
        .expect("Failed to set up BannerApi session");

    let banner_api_arc = Arc::new(banner_api);
    let app_state = AppState::new(banner_api_arc.clone(), &config.redis_url)
        .expect("Failed to create AppState");

    // Create BannerState for web service
    let banner_state = BannerState {
        api: banner_api_arc,
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
                        tracing::error!("Fatal error while sending error message: {}", e);
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

    service_manager.register_service("bot", bot_service);
    service_manager.register_service("web", web_service);

    // Spawn all registered services
    service_manager.spawn_all();

    // Set up CTRL+C signal handling
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("received ctrl+c, gracefully shutting down...");
    };

    // Main application loop - wait for services or CTRL+C
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
            // User requested shutdown
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
    }

    info!(exit_code, "application shutdown complete");
    std::process::exit(exit_code);
}
