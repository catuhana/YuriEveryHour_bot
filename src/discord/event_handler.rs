use std::sync::Arc;

use serenity::all::{ChannelId, Interaction, MessageId, UserId};
use serenity::builder::{CreateEmbed, CreateEmbedAuthor, EditMessage};
use serenity::client::{Context, EventHandler};
use serenity::http::CacheHttp;
use serenity::model::gateway::Ready;
use serenity::model::{Colour, Timestamp};
use tokio::time::Duration;

use super::interactions::yuri::YuriCInteraction;
use super::interactions::{register_interactions, run_interactions};
use super::{YuriDiscord, YuriState};

pub struct Handler {
    pub state: Arc<YuriState>,
}

impl Handler {
    pub async fn check_and_register_pending_approvals(
        &self,
        context: &Context,
    ) -> anyhow::Result<()> {
        debug!("checking and registering pending approvals");

        let pending_approvals = sqlx::query!("SELECT * FROM pending_approvals")
            .fetch_all(&self.state.database)
            .await?;

        for pending_approval in pending_approvals {
            let mut message = ChannelId::new(self.state.channels.approve_id)
                .message(
                    context.http(),
                    MessageId::new(pending_approval.message_id.try_into()?),
                )
                .await?;

            let submission = sqlx::query!(
                "SELECT user_id, artist, art_link, additional_information, sample_image_url FROM submissions WHERE submission_id = $1",
                pending_approval.submission_id
            )
            .fetch_one(&self.state.database)
            .await?;

            let embed = CreateEmbed::new()
                .author(CreateEmbedAuthor::new(format!(
                    "Submitted by {}",
                    UserId::new(submission.user_id.try_into()?)
                        .to_user(context.http())
                        .await?
                        .tag()
                )))
                .timestamp(Timestamp::now())
                .fields(vec![
                    ("Artist", submission.artist, true),
                    ("Art Link", submission.art_link, true),
                    (
                        "Additional Information",
                        submission
                            .additional_information
                            .unwrap_or_else(|| String::from("None")),
                        false,
                    ),
                ])
                .image(submission.sample_image_url.unwrap_or_default());

            let remaining_time = time::PrimitiveDateTime::new(
                time::OffsetDateTime::now_utc().date(),
                time::OffsetDateTime::now_utc().time(),
            ) - pending_approval.date;
            if remaining_time.whole_days() >= 1 {
                sqlx::query!(
                    "DELETE FROM pending_approvals WHERE submission_id = $1",
                    pending_approval.submission_id
                )
                .execute(&self.state.database)
                .await?;

                message
                    .edit(
                        context.http(),
                        EditMessage::new()
                            .content("Expired Submission")
                            .embed(embed.clone().colour(Colour::RED)),
                    )
                    .await?;

                continue;
            }

            YuriCInteraction::create_submission_collector(
                context,
                &self.state,
                pending_approval.submission_id,
                Duration::from_secs(remaining_time.whole_seconds().try_into()?),
                message,
                embed,
            )
            .await?;
        }

        debug!("checked and registered pending approvals");

        Ok(())
    }
}

#[async_trait::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, context: &Context, ready: &Ready) {
        info!("Connected to Discord as {}", ready.user.name);

        register_interactions(self.state.server_id, context).await;

        if let Err(error) = self.check_and_register_pending_approvals(context).await {
            error!(
                "an error occurred while checking and registering pending approvals: {error:#?}"
            );
        }
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
