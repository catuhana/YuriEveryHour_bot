// TODO: Silly-ify responses of the bot!

use std::sync::Arc;

use serenity::{
    all::{GatewayIntents, GuildId},
    Client,
};
use sqlx::PgPool;

use crate::config::{DiscordChannelConfig, DiscordConfig};

mod event_handler;
mod interactions;

pub struct YuriDiscord {
    token: String,
    state: Arc<YuriState>,
}

pub struct YuriState {
    pub server_id: GuildId,
    pub channels: DiscordChannelConfig,
    pub database: PgPool,
}

impl YuriDiscord {
    pub fn new(config: DiscordConfig, database: PgPool) -> Self {
        Self {
            token: config.token,
            state: Arc::new(YuriState {
                channels: config.channel,
                database,
                server_id: config.server_id.into(),
            }),
        }
    }

    pub async fn spawn(&self) -> anyhow::Result<()> {
        let mut client = Client::builder(&self.token, GatewayIntents::empty())
            .event_handler(event_handler::Handler {
                state: self.state.clone(),
            })
            .await?;

        debug!("spawning Discord bot");
        tokio::spawn(async move { client.start().await }).await??;

        Ok(())
    }
}
