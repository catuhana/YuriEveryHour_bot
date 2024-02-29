// TODO: Silly-ify responses of the bot!

use std::sync::Arc;

use serenity::{
    all::{GatewayIntents, GuildId, UserId},
    Client,
};
use sqlx::PgPool;
use tokio::sync::Mutex;

use crate::config::{DiscordChannelConfig, DiscordConfig};

use self::data::YuriData;

mod data;
mod event_handler;
mod handlers;
mod interactions;

pub struct YuriDiscord {
    token: String,
    state: Arc<YuriState>,
}

pub struct YuriState {
    pub database: PgPool,
    pub config: YuriConfig,
    pub data: Arc<Mutex<YuriData>>,
}

pub struct YuriConfig {
    pub channels: DiscordChannelConfig,
    pub team: Vec<UserId>,
    pub server_id: GuildId,
}

impl YuriDiscord {
    pub fn new(discord_config: DiscordConfig, database: PgPool) -> Self {
        Self {
            token: discord_config.token,
            state: YuriState {
                database,
                config: YuriConfig {
                    channels: discord_config.channels,
                    team: discord_config.team.iter().map(|id| (*id).into()).collect(),
                    server_id: discord_config.server_id.into(),
                },
                data: Arc::new(Mutex::new(YuriData {
                    pending_approvals: Vec::default(),
                })),
            }
            .into(),
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
