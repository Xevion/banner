use serde::Deserialize;
use serenity::all::{ClientBuilder, GatewayIntents};
use std::time::Duration;
use tokio::{signal, sync::broadcast, task::JoinSet};
use tracing::{error, info, warn};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::bot::{Data, age};
use figment::{Figment, providers::Env};

#[derive(Deserialize)]
struct Config {
    bot_token: String,
    database_url: String,
    redis_url: String,
    banner_base_url: String,
    bot_target_guild: u64,
    bot_app_id: u64,
}

mod bot;

#[derive(Debug)]
enum ServiceResult {
    GracefulShutdown,
    NormalCompletion,
    Error(Box<dyn std::error::Error + Send + Sync>),
}

/// Common trait for all services in the application
#[async_trait::async_trait]
trait Service: Send + Sync {
    /// The name of the service for logging
    fn name(&self) -> &'static str;

    /// Run the service's main work loop
    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// Gracefully shutdown the service
    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Generic service runner that handles the lifecycle
async fn run_service(
    mut service: Box<dyn Service>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> ServiceResult {
    let name = service.name();
    info!(service = name, "Service started");

    let work = async {
        match service.run().await {
            Ok(()) => {
                warn!(service = name, "Service completed unexpectedly");
                ServiceResult::NormalCompletion
            }
            Err(e) => {
                error!(service = name, "Service failed: {e}");
                ServiceResult::Error(e)
            }
        }
    };

    tokio::select! {
        result = work => result,
        _ = shutdown_rx.recv() => {
            info!(service = name, "Shutting down...");
            let start_time = std::time::Instant::now();

            match service.shutdown().await {
                Ok(()) => {
                    let elapsed = start_time.elapsed();
                    info!(service = name, "Shutdown completed in {elapsed:.2?}");
                    ServiceResult::GracefulShutdown
                }
                Err(e) => {
                    let elapsed = start_time.elapsed();
                    error!(service = name, "Shutdown failed after {elapsed:.2?}: {e}");
                    ServiceResult::Error(e)
                }
            }
        }
    }
}

/// Shutdown coordinator for managing graceful shutdown of multiple services
struct ShutdownCoordinator {
    shutdown_tx: broadcast::Sender<()>,
}

impl ShutdownCoordinator {
    fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self { shutdown_tx }
    }

    fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
}

/// Discord bot service implementation
struct BotService {
    client: serenity::Client,
    shard_manager: std::sync::Arc<serenity::gateway::ShardManager>,
}

impl BotService {
    fn new(client: serenity::Client) -> Self {
        let shard_manager = client.shard_manager.clone();
        Self {
            client,
            shard_manager,
        }
    }
}

#[async_trait::async_trait]
impl Service for BotService {
    fn name(&self) -> &'static str {
        "bot"
    }

    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match self.client.start().await {
            Ok(()) => {
                warn!(service = "bot", "Stopped early.");
                Err("bot stopped early".into())
            }
            Err(e) => {
                error!(service = "bot", "Error: {e:?}");
                Err(e.into())
            }
        }
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.shard_manager.shutdown_all().await;
        Ok(())
    }
}

/// Dummy service implementation for demonstration
struct DummyService {
    name: &'static str,
}

impl DummyService {
    fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[async_trait::async_trait]
impl Service for DummyService {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn run(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut counter = 0;
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            counter += 1;
            info!(service = self.name, "Service heartbeat ({counter})");

            // Simulate service failure after 60 seconds for demo
            if counter >= 6 {
                error!(service = self.name, "Service encountered an error");
                return Err("Service error".into());
            }
        }
    }

    async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Simulate cleanup work
        tokio::time::sleep(Duration::from_millis(3500)).await;
        Ok(())
    }
}

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
        let coordinator = shutdown_coordinator.shutdown_tx.clone();
        tokio::spawn(async move {
            signal::ctrl_c()
                .await
                .expect("Failed to install CTRL+C signal handler");
            info!("Received CTRL+C, initiating shutdown...");
            let _ = coordinator.send(());
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

    let service_result = match first_completion {
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
            service_result
        }
        Some(Ok(Err(e))) => {
            error!("Service task panicked: {e}");
            exit_code = 1;
            ServiceResult::Error("Task panic".into())
        }
        Some(Err(e)) => {
            error!("JoinSet error: {e}");
            exit_code = 1;
            ServiceResult::Error("JoinSet error".into())
        }
        None => {
            warn!("No services running");
            exit_code = 1;
            ServiceResult::Error("No services".into())
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
