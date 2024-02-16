use serenity::{
    all::{CommandInteraction, ResolvedOption},
    builder::{CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage},
    client::Context,
    http::CacheHttp,
};
use sqlx::PgPool;

use super::YuriInteraction;

pub struct PingInteraction;
impl YuriInteraction for PingInteraction {
    fn register() -> CreateCommand<'static> {
        CreateCommand::new("ping").description("Pong!")
    }

    async fn run(
        context: &Context,
        _database: PgPool,
        interaction: &CommandInteraction,
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
