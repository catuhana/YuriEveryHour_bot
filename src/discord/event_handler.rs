use std::sync::Arc;

use serenity::all::Interaction;
use serenity::client::{Context, EventHandler};
use serenity::model::gateway::Ready;

use super::interactions::{register_interactions, run_interactions};
use super::{YuriDiscord, YuriState};

pub struct Handler {
    pub state: Arc<YuriState>,
}

#[async_trait::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: &Context, ready: &Ready) {
        info!("Connected to Discord as {}", ready.user.name);

        register_interactions(self.state.server_id, context).await;
    }

    async fn interaction_create(&self, context: &Context, interaction: &Interaction) {
        if let Interaction::Command(command) = interaction {
            if let Err(error) = run_interactions(
                command.data.name.as_str(),
                context,
                command,
                self.state.clone(),
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
            state: discord.state,
        }
    }
}
