use serenity::all::{GuildId, Interaction};
use serenity::client::{Context, EventHandler};
use serenity::model::gateway::Ready;
use sqlx::PgPool;

use super::interactions::{register_interactions, run_interactions};
use super::YuriDiscord;

pub struct Handler {
    pub server_id: GuildId,
    pub database: PgPool,
}

#[async_trait::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: &Context, ready: &Ready) {
        info!("Connected to Discord as {}", ready.user.name);

        register_interactions(self.server_id, context).await;
    }

    async fn interaction_create(&self, context: &Context, interaction: &Interaction) {
        if let Interaction::Command(command) = interaction {
            if let Err(error) = run_interactions(
                command.data.name.as_str(),
                context,
                self.database.clone(),
                command,
                &command.data.options(),
            )
            .await
            {
                error!(
                    "an error occurred while running `{}` interaction: {error:#?}",
                    command.data.name
                )
            }
        }
    }
}

impl From<YuriDiscord> for Handler {
    fn from(discord: YuriDiscord) -> Self {
        Self {
            server_id: discord.server_id,
            database: discord.database,
        }
    }
}
