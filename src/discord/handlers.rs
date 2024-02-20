use serenity::all::{
    CacheHttp, Colour, ComponentInteraction, Context, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, EditMessage,
};

use crate::models::{
    pending_approvals::{PendingApprovalsHelpers, RemovePendingApproval},
    submissions::{Submission, SubmissionHelpers, SubmissionIds},
};

use super::event_handler::Handler;

impl Handler {
    pub async fn handle_approvals(
        &self,
        interaction: &ComponentInteraction,
        context: &Context,
    ) -> anyhow::Result<()> {
        debug!(
            "handling an approval with: `message_id`: {}",
            interaction.message.id
        );

        let pending_approvals = &mut self.state.data.write().await.pending_approvals;

        if let Some(pending_approval) = pending_approvals.iter().find(|pending_approval| {
            pending_approval.message_id == interaction.message.id.get() as i64
        }) {
            if chrono::Utc::now()
                .naive_utc()
                .signed_duration_since(pending_approval.date)
                .num_days()
                >= 1
            {
                pending_approvals
                    .remove_pending_approval(
                        &self.state.database,
                        RemovePendingApproval::MessageId(interaction.message.id.get()),
                    )
                    .await?;

                interaction
                    .create_response(
                        context.http(),
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("This approval has been expired and removed.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;

                return Ok(());
            }

            let mut pending_approval_message = context
                .http()
                .get_message(
                    interaction.channel_id,
                    (pending_approval.message_id as u64).into(),
                )
                .await?;

            if self.state.config.team.contains(&interaction.user.id) {
                let embed = interaction
                    .message
                    .embeds
                    .first()
                    .map(|embed| CreateEmbed::from(embed.clone()))
                    .expect("that message supposed to have an embed, but here we are.");

                if &interaction.data.custom_id.to_string() == "approve" {
                    let mut tx = self.state.database.begin().await?;
                    Submission::approve_submission(
                        &mut *tx,
                        SubmissionIds::SubmissionId(pending_approval.submission_id),
                    )
                    .await?;
                    pending_approvals
                        .remove_pending_approval(
                            &mut *tx,
                            RemovePendingApproval::SubmissionId(pending_approval.submission_id),
                        )
                        .await?;
                    tx.commit().await?;

                    pending_approval_message
                        .edit(
                            context.http(),
                            EditMessage::new()
                                .embed(
                                    embed
                                        .title(format!("Approved by {}!", interaction.user.tag()))
                                        .colour(Colour::DARK_GREEN),
                                )
                                .components(vec![]),
                        )
                        .await?;
                } else {
                    let mut tx = self.state.database.begin().await?;
                    Submission::reject_submission(
                        &mut *tx,
                        SubmissionIds::SubmissionId(pending_approval.submission_id),
                    )
                    .await?;
                    pending_approvals
                        .remove_pending_approval(
                            &mut *tx,
                            RemovePendingApproval::SubmissionId(pending_approval.submission_id),
                        )
                        .await?;
                    tx.commit().await?;

                    pending_approval_message
                        .edit(
                            context.http(),
                            EditMessage::new()
                                .embed(
                                    embed
                                        .title(format!("Rejected by {}!", interaction.user.tag()))
                                        .colour(Colour::RED),
                                )
                                .components(vec![]),
                        )
                        .await?;
                }
            } else {
                interaction
                    .create_response(
                        context.http(),
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("You don't have enough permissions to do that.")
                                .ephemeral(true),
                        ),
                    )
                    .await?;
            }
        } else {
            interaction
                .create_response(
                    context.http(),
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("This approval does not exist.")
                            .ephemeral(true),
                    ),
                )
                .await?;
        }

        debug!(
            "handled an approval with `message_id`: {}",
            interaction.message.id
        );
        Ok(())
    }
}
