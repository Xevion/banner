use super::Service;
use crate::bot::{Data, get_commands};
use crate::config::Config;
use crate::state::AppState;
use num_format::{Locale, ToFormattedString};
use serenity::Client;
use serenity::all::{ActivityData, ClientBuilder, GatewayIntents};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, warn};

/// Discord bot service implementation
pub struct BotService {
    client: Client,
    shard_manager: Arc<serenity::gateway::ShardManager>,
}

impl BotService {
    /// Create a new Discord bot client with full configuration
    pub async fn create_client(
        config: &Config,
        app_state: AppState,
    ) -> Result<Client, anyhow::Error> {
        let intents = GatewayIntents::non_privileged();
        let bot_target_guild = config.bot_target_guild;

        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: get_commands(),
                pre_command: |ctx| {
                    Box::pin(async move {
                        let content = match ctx {
                            poise::Context::Application(_) => ctx.invocation_string(),
                            poise::Context::Prefix(prefix) => prefix.msg.content.to_string(),
                        };
                        let channel_name = ctx
                            .channel_id()
                            .name(ctx.http())
                            .await
                            .unwrap_or("unknown".to_string());

                        let span = tracing::Span::current();
                        span.record("command_name", ctx.command().qualified_name.as_str());
                        span.record("invocation", ctx.invocation_string());
                        span.record("msg.content", content.as_str());
                        span.record("msg.author", ctx.author().tag().as_str());
                        span.record("msg.id", ctx.id());
                        span.record("msg.channel_id", ctx.channel_id().get());
                        span.record("msg.channel", channel_name.as_str());

                        tracing::info!(
                            command_name = ctx.command().qualified_name.as_str(),
                            invocation = ctx.invocation_string(),
                            msg.content = %content,
                            msg.author = %ctx.author().tag(),
                            msg.author_id = %ctx.author().id,
                            msg.id = %ctx.id(),
                            msg.channel = %channel_name.as_str(),
                            msg.channel_id = %ctx.channel_id(),
                            "{} invoked by {}",
                            ctx.command().name,
                            ctx.author().tag()
                        );
                    })
                },
                on_error: |error| {
                    Box::pin(async move {
                        if let Err(e) = poise::builtins::on_error(error).await {
                            tracing::error!(error = %e, "Fatal error while sending error message");
                        }
                    })
                },
                ..Default::default()
            })
            .setup(move |ctx, _ready, framework| {
                let app_state = app_state.clone();
                Box::pin(async move {
                    poise::builtins::register_in_guild(
                        ctx,
                        &framework.options().commands,
                        bot_target_guild.into(),
                    )
                    .await?;
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                    // Start status update task
                    Self::start_status_update_task(ctx.clone(), app_state.clone()).await;

                    Ok(Data { app_state })
                })
            })
            .build();

        Ok(ClientBuilder::new(config.bot_token.clone(), intents)
            .framework(framework)
            .await?)
    }

    /// Start the status update task for the Discord bot
    async fn start_status_update_task(ctx: serenity::client::Context, app_state: AppState) {
        tokio::spawn(async move {
            let max_interval = Duration::from_secs(300); // 5 minutes
            let base_interval = Duration::from_secs(30);
            let mut interval = tokio::time::interval(base_interval);
            let mut previous_course_count: Option<i64> = None;

            // This runs once immediately on startup, then with adaptive intervals
            loop {
                interval.tick().await;

                // Get the course count, update the activity if it has changed/hasn't been set this session
                let course_count = app_state.get_course_count().await.unwrap();
                if previous_course_count.is_none() || previous_course_count != Some(course_count) {
                    ctx.set_activity(Some(ActivityData::playing(format!(
                        "Querying {:} classes",
                        course_count.to_formatted_string(&Locale::en)
                    ))));
                }

                // Increase or reset the interval
                interval = tokio::time::interval(
                    // Avoid logging the first 'change'
                    if course_count != previous_course_count.unwrap_or(0) {
                        if previous_course_count.is_some() {
                            debug!(
                                new_course_count = course_count,
                                last_interval = interval.period().as_secs(),
                                "Course count changed, resetting interval"
                            );
                        }

                        // Record the new course count
                        previous_course_count = Some(course_count);

                        // Reset to base interval
                        base_interval
                    } else {
                        // Increase interval by 10% (up to maximum)
                        let new_interval = interval.period().mul_f32(1.1).min(max_interval);
                        debug!(
                            current_course_count = course_count,
                            last_interval = interval.period().as_secs(),
                            new_interval = new_interval.as_secs(),
                            "Course count unchanged, increasing interval"
                        );

                        new_interval
                    },
                );

                // Reset the interval, otherwise it will tick again immediately
                interval.reset();
            }
        });
    }

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
                warn!(service = "bot", "stopped early");
                Err(anyhow::anyhow!("bot stopped early"))
            }
            Err(e) => {
                error!(service = "bot", "error: {e:?}");
                Err(e.into())
            }
        }
    }

    async fn shutdown(&mut self) -> Result<(), anyhow::Error> {
        self.shard_manager.shutdown_all().await;
        Ok(())
    }
}
