use std::sync::Arc;

use serenity::{
    all::{CommandInteraction, ResolvedOption},
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage},
    client::Context,
    http::CacheHttp,
};

use crate::discord::YuriState;

use super::YuriInteraction;

pub struct PingInteraction;
impl YuriInteraction for PingInteraction {
    fn register() -> CreateCommand<'static> {
        CreateCommand::new("ping").description("Pong!")
    }

    async fn run(
        context: &Context,
        interaction: &CommandInteraction,
        _state: Arc<YuriState>,
        _options: &[ResolvedOption<'_>],
    ) -> anyhow::Result<()> {
        interaction
            .create_response(
                context.http(),
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("Pong!"),
                ),
            )
            .await?;

        Ok(())
    }
}
