// TODO: Silly-ify responses of the bot!

use serenity::{
    all::{GatewayIntents, GuildId},
    Client,
};

use crate::config::DiscordConfig;

mod event_handler;
mod interactions;

pub struct YuriDiscord {
    token: String,
    server_id: GuildId,
}

impl YuriDiscord {
    pub async fn spawn(&self) -> anyhow::Result<()> {
        let mut client = Client::builder(&self.token, GatewayIntents::empty())
            .event_handler(event_handler::Handler {
                server_id: self.server_id,
            })
            .await?;

        debug!("spawning Discord bot");
        tokio::spawn(async move { client.start().await }).await??;

        Ok(())
    }
}

impl From<DiscordConfig> for YuriDiscord {
    fn from(config: DiscordConfig) -> Self {
        Self {
            token: config.token,
            server_id: GuildId::new(config.server_id),
        }
    }
}
