use std::sync::Arc;

use serenity::{
    all::{CommandInteraction, CommandOptionType, ResolvedOption, ResolvedValue},
    builder::{
        CreateCommand, CreateCommandOption, CreateInteractionResponse,
        CreateInteractionResponseMessage,
    },
    client::Context,
    http::CacheHttp,
    utils::CreateQuickModal,
};

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
                    .paragraph_field("Additional Context"),
            )
            .await?
        {
            let (artist, art_link, additional_context) = {
                let inputs = modal_response.inputs.clone();
                (
                    inputs[0].to_string(),
                    inputs[1].to_string(),
                    inputs[2].to_string(),
                )
            };
            let sample_image_url = {
                if let Some(ResolvedOption {
                    value: ResolvedValue::Attachment(sample),
                    ..
                }) = options.first()
                {
                    Some(sample.url.to_string())
                } else {
                    None
                }
            };
            let submitter_id = i64::from(interaction.user.id);

            if let Err(error) = sqlx::query!(
                r#"
                INSERT INTO submissions(user_id, artist, art_link, additional_context, sample_image_url)
                VALUES($1, $2, $3, $4, $5)
            "#, submitter_id, artist, art_link, additional_context, sample_image_url
            )
            .execute(&state.database)
            .await {
                error!("error submitting Yuri addition: {:#?}", error);
                modal_response.interaction.create_response(
                    context.http(),
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("An error occurred while submitting your Yuri addition. Please try again later.")
                            .ephemeral(true),
                    ),
                ).await?;
            } else {
                modal_response
                    .interaction
                    .create_response(
                        context.http(),
                        CreateInteractionResponse::Message(
                            CreateInteractionResponseMessage::new()
                                .content("Submitted Yuri addition! After being checked, it will be posted to the votes channel for public approval.")
                                .ephemeral(true),
                        ),
                    )
                    .await?
            }
        }

        Ok(())
    }
}
