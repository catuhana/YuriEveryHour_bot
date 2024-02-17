use std::{sync::Arc, time::Duration};

use serenity::{
    all::{
        ButtonStyle, ChannelId, CommandInteraction, CommandOptionType, ComponentInteractionDataKind, Mention, ResolvedOption, ResolvedValue
    },
    builder::{
        CreateActionRow, CreateButton, CreateCommand, CreateCommandOption, CreateEmbed,
        CreateEmbedAuthor, CreateInteractionResponse, CreateInteractionResponseMessage,
        CreateMessage, EditMessage,
    },
    client::Context,
    http::CacheHttp,
    model::{Colour, Timestamp},
    utils::CreateQuickModal,
};
use tokio::task::JoinHandle;

use crate::discord::YuriState;

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
            let (artist, art_link, additional_information, sample_image) = {
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

            match sqlx::query!(
                "INSERT INTO submissions(user_id, artist, art_link, additional_information, sample_image_url) VALUES($1, $2, $3, $4, $5) RETURNING submission_id",
                i64::from(interaction.user.id),
                artist,
                art_link,
                additional_information,
                sample_image
            )
                .fetch_one(&state.database)
                .await
            {
                Ok(submission_table) => {
                    let embed: Arc<CreateEmbed> = CreateEmbed::new()
                        .author(CreateEmbedAuthor::new(format!(
                            "Submitted by {}",
                            interaction.user.tag()
                        )))
                        .timestamp(Timestamp::now())
                        .fields(vec![
                            ("Artist", artist, true),
                            ("Art Link", art_link, true),
                            ("Additional Information", additional_information, false),
                        ])
                        .image(sample_image.unwrap_or_default())
                        .into();

                    let mut submission_approval = ChannelId::new(state.channels.approve_id)
                        .send_message(
                            context.http(),
                            CreateMessage::new()
                                .content("New Yuri Submission!")
                                .embed((*embed).clone())
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

                        let task_context: Arc<Context> = context.clone().into();
                        let task_state = state.clone();
                        let _task: JoinHandle<anyhow::Result<()>> = tokio::spawn(async move {
                            loop {
                                if let Some(submission_choice) = match submission_approval
                                    .await_component_interaction(task_context.shard.clone())
                                    .timeout(Duration::from_secs(86_400))
                                    .await
                                {
                                    Some(interaction) => {
                                        if task_state.team.iter().any(|id| *id == interaction.user.id.get()) {
                                            match &interaction.data.kind {
                                                ComponentInteractionDataKind::Button => {
                                                    Some(interaction.data.custom_id.to_string())
                                                }
                                                _ => None,
                                            }
                                        } else {
                                            interaction
                                                .create_response(
                                                    task_context.http(),
                                                    CreateInteractionResponse::Message(
                                                        CreateInteractionResponseMessage::new()
                                                            .content("You don't have enough permissions to do that.")
                                                            .ephemeral(true),
                                                    ),
                                                )
                                                .await?;
                                            continue;
                                        }
                                    }
                                    None => break,
                                } {
                                    if &submission_choice == "approve" {
                                        sqlx::query!("UPDATE submissions SET decision = 'approved', pending_approval = FALSE, submission_decision_date = NOW() WHERE submission_id = $1", submission_table.submission_id)
                                            .execute(&task_state.database)
                                            .await?;

                                        submission_approval
                                            .edit(
                                                task_context.http(),
                                                EditMessage::new()
                                                    .embed(
                                                        (*embed)
                                                            .clone()
                                                            .title("Approved!")
                                                            .colour(Colour::DARK_GREEN),
                                                    )
                                                    .components(vec![]),
                                            )
                                            .await?;
                                    } else {
                                        sqlx::query!("UPDATE submissions SET decision = 'rejected', pending_approval = FALSE, submission_decision_date = NOW() WHERE submission_id = $1", submission_table.submission_id)
                                            .execute(&task_state.database)
                                            .await?;

                                        submission_approval
                                            .edit(
                                                task_context.http(),
                                                EditMessage::new()
                                                    .embed((*embed).clone().title("Rejected!").colour(Colour::DARK_RED))
                                                    .components(vec![]),
                                            )
                                            .await?;
                                    }
                                };
                            }
                        
                            Ok(())
                        });

                        modal_response
                            .interaction
                            .create_response(
                                context.http(),
                                CreateInteractionResponse::Message(
                                    CreateInteractionResponseMessage::new()
                                        .content(format!("Your Yuri addition has been submitted for review! After being approved, it will be posted to {} for public voting.", Mention::from(ChannelId::new(state.channels.vote_id))))
                                        .ephemeral(true),
                                ),
                            )
                            .await?;
                    },
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
