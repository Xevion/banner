use super::Service;
use crate::web::{BannerState, create_router};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

/// Web server service implementation
pub struct WebService {
    port: u16,
    banner_state: BannerState,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl WebService {
    pub fn new(port: u16, banner_state: BannerState) -> Self {
        Self {
            port,
            banner_state,
            shutdown_tx: None,
        }
    }
}

#[async_trait::async_trait]
impl Service for WebService {
    fn name(&self) -> &'static str {
        "web"
    }

    async fn run(&mut self) -> Result<(), anyhow::Error> {
        // Create the main router with Banner API routes
        let app = create_router(self.banner_state.clone());

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        info!(
            service = "web",
            link = format!("http://localhost:{}", addr.port()),
            "starting web server",
        );

        let listener = TcpListener::bind(addr).await?;
        debug!(
            service = "web",
            "web server listening on {}",
            format!("http://{}", addr)
        );

        // Create internal shutdown channel for axum graceful shutdown
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Use axum's graceful shutdown with the internal shutdown signal
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.recv().await;
                debug!(
                    service = "web",
                    "received shutdown signal, starting graceful shutdown"
                );
            })
            .await?;

        info!(service = "web", "web server stopped");
        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        } else {
            warn!(
                service = "web",
                "no shutdown channel found, cannot trigger graceful shutdown"
            );
        }
        Ok(())
    }
}
