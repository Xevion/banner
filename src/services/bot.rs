use super::{Service, ServiceResult};
use serenity::Client;
use std::sync::Arc;
use tracing::{error, warn};

/// Discord bot service implementation
pub struct BotService {
    client: Client,
    shard_manager: Arc<serenity::gateway::ShardManager>,
}

impl BotService {
    pub fn new(client: Client) -> Self {
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

    async fn run(&mut self) -> Result<(), anyhow::Error> {
        match self.client.start().await {
            Ok(()) => {
                warn!(service = "bot", "Stopped early.");
                Err(anyhow::anyhow!("bot stopped early"))
            }
            Err(e) => {
                error!(service = "bot", "Error: {e:?}");
                Err(e.into())
            }
        }
    }

    async fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        self.shard_manager.shutdown_all().await;
        Ok(())
    }
}
