use serenity::all::{ClientBuilder, GatewayIntents};
use tokio::signal;
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::bot::{Data, age};
use crate::config::Config;
use crate::services::manager::ServiceManager;
use crate::services::{ServiceResult, bot::BotService, dummy::DummyService, run_service};
use figment::{Figment, providers::Env};

mod bot;
mod config;
mod services;

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
    .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let config: Config = Figment::new()
        .merge(Env::prefixed("APP_"))
        .extract()
        .expect("Failed to load config");

    // Configure the client with your Discord bot token in the environment.
    let intents = GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = ClientBuilder::new(config.bot_token, intents)
        .framework(framework)
        .await
        .expect("Failed to build client");

    // Extract shutdown timeout before moving config
    let shutdown_timeout = config.shutdown_timeout;

    // Create service manager
    let mut service_manager = ServiceManager::new();

    // Create and add services
    let bot_service = Box::new(BotService::new(client));
    let dummy_service = Box::new(DummyService::new("background"));

    let bot_handle = tokio::spawn(run_service(bot_service, service_manager.subscribe()));
    let dummy_handle = tokio::spawn(run_service(dummy_service, service_manager.subscribe()));

    service_manager.add_service("bot".to_string(), bot_handle);
    service_manager.add_service("background".to_string(), dummy_handle);

    // Set up CTRL+C signal handling
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Received CTRL+C, gracefully shutting down...");
    };

    // Main application loop - wait for services or CTRL+C
    let mut exit_code = 0;

    let join = |strings: Vec<String>| {
        strings
            .iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(", ")
    };

    tokio::select! {
        (service_name, result) = service_manager.run() => {
            // A service completed unexpectedly
            match result {
                ServiceResult::GracefulShutdown => {
                    info!(service = service_name, "Service completed gracefully");
                }
                ServiceResult::NormalCompletion => {
                    warn!(service = service_name, "Service completed unexpectedly");
                    exit_code = 1;
                }
                ServiceResult::Error(e) => {
                    error!(service = service_name, "Service failed: {e}");
                    exit_code = 1;
                }
            }

            // Shutdown remaining services
            match service_manager.shutdown(shutdown_timeout).await {
                Ok(()) => {
                    info!("Graceful shutdown complete");
                }
                Err(pending_services) => {
                    warn!(
                        "Graceful shutdown elapsed - the following service(s) did not complete: {}",
                        join(pending_services)
                    );
                    exit_code = if exit_code == 0 { 2 } else { exit_code };
                }
            }
        }
        _ = ctrl_c => {
            // User requested shutdown
            match service_manager.shutdown(shutdown_timeout).await {
                Ok(()) => {
                    info!("Graceful shutdown complete");
                }
                Err(pending_services) => {
                    warn!(
                        "Graceful shutdown elapsed - the following service(s) did not complete: {}",
                        join(pending_services)
                    );
                    exit_code = 2;
                }
            }
        }
    }

    info!("Application shutdown complete (exit code: {})", exit_code);
    std::process::exit(exit_code);
}
