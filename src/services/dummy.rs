use super::Service;
use std::time::Duration;
use tracing::{error, info};

/// Dummy service implementation for demonstration
pub struct DummyService {
    name: &'static str,
}

impl DummyService {
    pub fn new(name: &'static str) -> Self {
        Self { name }
    }
}

#[async_trait::async_trait]
impl Service for DummyService {
    fn name(&self) -> &'static str {
        self.name
    }

    async fn run(&mut self) -> Result<(), anyhow::Error> {
        let mut counter = 0;
        loop {
            tokio::time::sleep(Duration::from_secs(10)).await;
            counter += 1;
            info!(service = self.name, "Service heartbeat ({counter})");

            // Simulate service failure after 60 seconds for demo
            if counter >= 6 {
                error!(service = self.name, "Service encountered an error");
                return Err(anyhow::anyhow!("Service error"));
            }
        }
    }

    async fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        // Simulate cleanup work
        tokio::time::sleep(Duration::from_millis(6000)).await;
        Ok(())
    }
}
