use std::sync::Arc;

use serenity::{
    all::{
        ButtonStyle, ChannelId, CommandInteraction, CommandOptionType, Mention, ResolvedOption,
        ResolvedValue,
    },
    builder::{
        CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateEmbed,
        CreateEmbedAuthor, CreateInteractionResponse, CreateInteractionResponseMessage,
        CreateMessage,
    },
    client::Context,
    http::CacheHttp,
    model::Timestamp,
    utils::CreateQuickModal,
};

use crate::{
    discord::YuriState,
    models::{
        pending_approvals::{AddPendingApproval, PendingApprovalsHelpers},
        submissions::{AddSubmission, Submission, SubmissionHelpers},
    },
};

use super::YuriInteraction;

pub struct YuriCInteraction;
impl YuriInteraction for YuriCInteraction {
    fn register() -> CreateCommand<'static> {
        CreateCommand::new("yuri")
            .description("Submit a form for Yuri content addition!")
            .add_option(CreateCommandOption::new(
                CommandOptionType::Attachment,
                "sample",
                "Sample of the content to be submitted to help us decide quicker!",
            ))
    }

    async fn run(
        context: &Context,
        interaction: &CommandInteraction,
        state: Arc<YuriState>,
        options: &[ResolvedOption<'_>],
    ) -> anyhow::Result<()> {
        if let Some(modal_response) = interaction
            .quick_modal(
                context,
                CreateQuickModal::new("Submit Yuri")
                    .short_field("Artist's Name or Link")
                    .short_field("Art's Link")
                    .paragraph_field("Additional Information"),
            )
            .await?
        {
            let (artist, art_link, additional_information, sample_image_url) = {
                let inputs = modal_response.inputs;

                (
                    inputs[0].to_string(),
                    inputs[1].to_string(),
                    inputs[2].to_string(),
                    options.first().and_then(|option| {
                        if let ResolvedValue::Attachment(attachment) = option.value {
                            Some(attachment.url.to_string())
                        } else {
                            None
                        }
                    }),
                )
            };

            match Submission::add_submission(
                &state.database,
                AddSubmission {
                    user_id: interaction.user.id.get(),
                    artist,
                    art_link,
                    additional_information: Some(additional_information),
                    sample_image_url,
                },
            )
            .await
            {
                Ok(submission_table) => {
                    let embed = CreateEmbed::new()
                        .author(CreateEmbedAuthor::new(format!(
                            "Submitted by {user_tag}",
                            user_tag = interaction.user.tag()
                        )))
                        .timestamp(Timestamp::now())
                        .fields(vec![
                            ("Artist", submission_table.artist, true),
                            ("Art Link", submission_table.art_link, true),
                            (
                                "Additional Information",
                                submission_table
                                    .additional_information
                                    .unwrap_or_else(|| String::from("*Not provided*")),
                                false,
                            ),
                        ])
                        .image(submission_table.sample_image_url.unwrap_or_default());

                    let submission_approval_message =
                        ChannelId::new(state.config.channels.approve_id)
                            .send_message(
                                context.http(),
                                CreateMessage::new()
                                    .content("New Yuri Submission!")
                                    .embed(embed.clone())
                                    .components(vec![CreateActionRow::Buttons(vec![
                                        CreateButton::new("approve")
                                            .label("Approve")
                                            .style(ButtonStyle::Success),
                                        CreateButton::new("reject")
                                            .label("Reject")
                                            .style(ButtonStyle::Danger),
                                    ])]),
                            )
                            .await?;

                    state
                        .data
                        .write()
                        .await
                        .pending_approvals
                        .add_pending_approval(
                            &state.database,
                            AddPendingApproval {
                                submission_id: submission_table.submission_id,
                                message_id: submission_approval_message.id.get(),
                            },
                        )
                        .await?;

                    modal_response
                            .interaction
                            .create_response(
                                context.http(),
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content(format!("Your Yuri addition has been submitted for review! After being approved, it will be posted to {vote_channel} for public voting.", vote_channel = Mention::from(ChannelId::new(state.config.channels.vote_id))))
                                        .ephemeral(true),
                                ),
                            )
                            .await?;
                }
                Err(error) => {
                    error!("error submitting Yuri addition: {:#?}", error);

                    modal_response.interaction.create_response(
                        context.http(),
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("An error occurred while submitting your Yuri addition, please try again.")
                                .ephemeral(true),
                        ),
                    ).await?;
                }
            }
        };

        Ok(())
    }
}
