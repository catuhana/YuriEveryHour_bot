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
        options: &[ResolvedOption<'_>],
    ) -> anyhow::Result<()> {
        if let Some(modal_response) = interaction
            .quick_modal(
                context,
                CreateQuickModal::new("Submit Yuri")
                    .short_field("Artist's Link")
                    .short_field("Content's Link")
                    .paragraph_field("Additional Context"),
            )
            .await?
        {
            let sample = {
                if let Some(ResolvedOption {
                    value: ResolvedValue::Attachment(sample),
                    ..
                }) = options.first()
                {
                    Some(sample)
                } else {
                    None
                }
            };

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
                .await?;

            dbg!("{:#?}{:#?}", sample, modal_response.inputs);
        }

        Ok(())
    }
}
