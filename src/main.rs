use serenity::all::{ClientBuilder, GatewayIntents};
use std::time::Duration;
use tokio::{signal, task::JoinSet};
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::bot::{Data, age};
use crate::config::Config;
use crate::services::{ServiceResult, bot::BotService, dummy::DummyService, run_service};
use crate::shutdown::ShutdownCoordinator;
use figment::{Figment, providers::Env};

mod bot;
mod config;
mod services;
mod shutdown;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // Configure logging
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn,banner=debug"));
    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();
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

    let shutdown_coordinator = ShutdownCoordinator::new();

    // Create services
    let bot_service = Box::new(BotService::new(client));
    let dummy_service = Box::new(DummyService::new("background"));

    // Start services using the unified runner
    let bot_handle = {
        let shutdown_rx = shutdown_coordinator.subscribe();
        tokio::spawn(run_service(bot_service, shutdown_rx))
    };

    let dummy_handle = {
        let shutdown_rx = shutdown_coordinator.subscribe();
        tokio::spawn(run_service(dummy_service, shutdown_rx))
    };

    // Set up signal handling
    let signal_handle = {
        let shutdown_tx = shutdown_coordinator.shutdown_tx();
        tokio::spawn(async move {
            signal::ctrl_c()
                .await
                .expect("Failed to install CTRL+C signal handler");
            info!("Received CTRL+C, initiating shutdown...");
            let _ = shutdown_tx.send(());
            ServiceResult::GracefulShutdown
        })
    };

    // Put all services in a JoinSet for unified handling
    let mut services = JoinSet::new();
    services.spawn(bot_handle);
    services.spawn(dummy_handle);
    services.spawn(signal_handle);

    // Wait for any service to complete or signal
    let mut exit_code = 0;
    let first_completion = services.join_next().await;

    match first_completion {
        Some(Ok(Ok(service_result))) => {
            // A service completed successfully
            match &service_result {
                ServiceResult::GracefulShutdown => {
                    // This means CTRL+C was pressed
                }
                ServiceResult::NormalCompletion => {
                    warn!("A service completed unexpectedly");
                    exit_code = 1;
                }
                ServiceResult::Error(e) => {
                    error!("Service failure: {e}");
                    exit_code = 1;
                }
            }
        }
        Some(Ok(Err(e))) => {
            error!("Service task panicked: {e}");
            exit_code = 1;
        }
        Some(Err(e)) => {
            error!("JoinSet error: {e}");
            exit_code = 1;
        }
        None => {
            warn!("No services running");
            exit_code = 1;
        }
    };

    // Signal all services to shut down
    shutdown_coordinator.shutdown();

    // Wait for graceful shutdown with timeout
    let remaining_count = services.len();
    if remaining_count > 0 {
        info!("Waiting for {remaining_count} remaining services to shutdown (5s timeout)...");
        let shutdown_result = tokio::time::timeout(Duration::from_secs(5), async {
            while let Some(result) = services.join_next().await {
                match result {
                    Ok(Ok(ServiceResult::GracefulShutdown)) => {
                        // Service shutdown logged by the service itself
                    }
                    Ok(Ok(ServiceResult::NormalCompletion)) => {
                        warn!("Service completed normally during shutdown");
                    }
                    Ok(Ok(ServiceResult::Error(e))) => {
                        error!("Service error during shutdown: {e}");
                    }
                    Ok(Err(e)) => {
                        error!("Service panic during shutdown: {e}");
                    }
                    Err(e) => {
                        error!("Service join error: {e}");
                    }
                }
            }
        })
        .await;

        match shutdown_result {
            Ok(()) => {
                info!("All services shutdown completed");
            }
            Err(_) => {
                warn!("Shutdown timeout - some services may not have completed");
                exit_code = if exit_code == 0 { 2 } else { exit_code };
            }
        }
    } else {
        info!("No remaining services to shutdown");
    }

    info!("Application shutdown complete (exit code: {})", exit_code);
    std::process::exit(exit_code);
}
