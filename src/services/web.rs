use super::Service;
use crate::web::create_router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{info, trace, warn};

/// Web server service implementation
pub struct WebService {
    port: u16,
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl WebService {
    pub fn new(port: u16) -> Self {
        Self {
            port,
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
        let app = create_router();

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));

        let listener = TcpListener::bind(addr).await?;
        info!(
            service = "web",
            address = %addr,
            link = format!("http://localhost:{}", addr.port()),
            "web server listening"
        );

        // Create internal shutdown channel for axum graceful shutdown
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Use axum's graceful shutdown with the internal shutdown signal
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.recv().await;
                trace!(
                    service = "web",
                    "received shutdown signal, starting graceful shutdown"
                );
            })
            .await?;

        trace!(service = "web", "graceful shutdown completed");
        info!(service = "web", "web server stopped");

        Ok(())
    }

    async fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
            trace!(service = "web", "sent shutdown signal to axum");
        } else {
            warn!(
                service = "web",
                "no shutdown channel found, cannot trigger graceful shutdown"
            );
        }
        Ok(())
    }
}
