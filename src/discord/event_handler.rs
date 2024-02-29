use std::sync::Arc;

use serenity::all::Interaction;
use serenity::client::{Context, EventHandler};
use serenity::model::gateway::Ready;

use crate::discord::interactions::register_interactions;
use crate::models::pending_approvals::{
    PendingApproval, PendingApprovalHelpers, PendingApprovalsHelpers,
};

use super::interactions::run_interactions;
use super::{YuriDiscord, YuriState};

pub struct Handler {
    pub state: Arc<YuriState>,
}

#[async_trait::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: &Context, ready: &Ready) {
        info!(
            "Connected to Discord as {username}",
            username = ready.user.name
        );

        register_interactions(self.state.config.server_id, context).await;

        {
            let pending_approvals = &mut self.state.data.lock().await.pending_approvals;

            if let Err(error) =
                PendingApproval::remove_expired_approvals(&self.state.database).await
            {
                error!("an error occurred while removing expired approvals: {error:#?}");
            }

            if let Err(error) = pending_approvals
                .populate_pending_approvals(&self.state.database)
                .await
            {
                error!("an error occurred while populating pending approvals: {error:#?}");
            };
        }
    }

    async fn interaction_create(&self, context: &Context, interaction: &Interaction) {
        match interaction {
            Interaction::Command(command) => {
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
                        "an error occurred while running `{interaction_name}` interaction: {error:#?}",
                        interaction_name = command.data.name
                    );
                }
            }
            Interaction::Component(component_interaction) => {
                if let Err(error) = &self.handle_approvals(component_interaction, context).await {
                    error!("an error occurred while handling approvals: {error:#?}");
                }
            }
            _ => {}
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
