use crate::app::App;
use crate::cli::{Args, ServiceName, determine_enabled_services};
use crate::logging::setup_logging;
use clap::Parser;
use std::process::ExitCode;
use tracing::info;

mod app;
mod banner;
mod bot;
mod cli;
mod config;
mod data;
mod error;
mod formatter;
mod logging;
mod scraper;
mod services;
mod signals;
mod state;
#[allow(dead_code)]
mod status;
mod web;

#[tokio::main]
async fn main() -> ExitCode {
    dotenvy::dotenv().ok();

    // Parse CLI arguments
    let args = Args::parse();

    // Determine which services should be enabled
    let enabled_services: Vec<ServiceName> =
        determine_enabled_services(&args).expect("Failed to determine enabled services");

    // Create and initialize the application
    let mut app = App::new().await.expect("Failed to initialize application");

    // Setup logging — must happen before any info!() calls to avoid silently dropped logs
    setup_logging(app.config(), args.tracing);

    info!(
        enabled_services = ?enabled_services,
        "services configuration loaded"
    );

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

    // Setup services (web, scraper)
    app.setup_services(&enabled_services)
        .expect("Failed to setup services");

    // Setup bot service if enabled
    if enabled_services.contains(&ServiceName::Bot) {
        app.setup_bot_service()
            .await
            .expect("Failed to setup bot service");
    }

    // Start all services and run the application
    app.start_services();
    app.run().await
}
