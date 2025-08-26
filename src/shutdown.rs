use tokio::sync::broadcast;

/// Shutdown coordinator for managing graceful shutdown of multiple services
pub struct ShutdownCoordinator {
    shutdown_tx: broadcast::Sender<()>,
}

impl ShutdownCoordinator {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self { shutdown_tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }

    pub fn shutdown_tx(&self) -> broadcast::Sender<()> {
        self.shutdown_tx.clone()
    }
}
