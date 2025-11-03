use crate::banner::BannerApi;
use crate::cli::ServiceName;
use crate::config::Config;
use crate::scraper::ScraperService;
use crate::services::bot::BotService;
use crate::services::manager::ServiceManager;
use crate::services::web::WebService;
use crate::state::AppState;
use crate::web::routes::BannerState;
use figment::value::UncasedStr;
use figment::{Figment, providers::Env};
use sqlx::postgres::PgPoolOptions;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info};

/// Main application struct containing all necessary components
pub struct App {
    config: Config,
    db_pool: sqlx::PgPool,
    banner_api: Arc<BannerApi>,
    app_state: AppState,
    banner_state: BannerState,
    service_manager: ServiceManager,
}

impl App {
    /// Create a new App instance with all necessary components initialized
    pub async fn new() -> Result<Self, anyhow::Error> {
        // Load configuration
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

        // Check if the database URL is via private networking
        let is_private = config.database_url.contains("railway.internal");
        let slow_threshold = Duration::from_millis(if is_private { 200 } else { 500 });

        // Create database connection pool
        let db_pool = PgPoolOptions::new()
            .min_connections(0)
            .max_connections(4)
            .acquire_slow_threshold(slow_threshold)
            .acquire_timeout(Duration::from_secs(4))
            .idle_timeout(Duration::from_secs(60 * 2))
            .max_lifetime(Duration::from_secs(60 * 30))
            .connect(&config.database_url)
            .await
            .expect("Failed to create database pool");

        info!(
            is_private = is_private,
            slow_threshold = format!("{:.2?}", slow_threshold),
            "database pool established"
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
        let banner_state = BannerState {};

        Ok(App {
            config,
            db_pool,
            banner_api: banner_api_arc,
            app_state,
            banner_state,
            service_manager: ServiceManager::new(),
        })
    }

    /// Setup and register services based on enabled service list
    pub fn setup_services(&mut self, services: &[ServiceName]) -> Result<(), anyhow::Error> {
        // Register enabled services with the manager
        if services.contains(&ServiceName::Web) {
            let web_service =
                Box::new(WebService::new(self.config.port, self.banner_state.clone()));
            self.service_manager
                .register_service(ServiceName::Web.as_str(), web_service);
        }

        if services.contains(&ServiceName::Scraper) {
            let scraper_service = Box::new(ScraperService::new(
                self.db_pool.clone(),
                self.banner_api.clone(),
            ));
            self.service_manager
                .register_service(ServiceName::Scraper.as_str(), scraper_service);
        }

        // Check if any services are enabled
        if !self.service_manager.has_services() && !services.contains(&ServiceName::Bot) {
            error!("No services enabled. Cannot start application.");
            return Err(anyhow::anyhow!("No services enabled"));
        }

        Ok(())
    }

    /// Setup bot service if enabled
    pub async fn setup_bot_service(&mut self) -> Result<(), anyhow::Error> {
        let client = BotService::create_client(&self.config, self.app_state.clone())
            .await
            .expect("Failed to create Discord client");
        let bot_service = Box::new(BotService::new(client));
        self.service_manager
            .register_service(ServiceName::Bot.as_str(), bot_service);
        Ok(())
    }

    /// Start all registered services
    pub fn start_services(&mut self) {
        self.service_manager.spawn_all();
    }

    /// Run the application and handle shutdown signals
    pub async fn run(self) -> ExitCode {
        use crate::signals::handle_shutdown_signals;
        handle_shutdown_signals(self.service_manager, self.config.shutdown_timeout).await
    }

    /// Get a reference to the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}
