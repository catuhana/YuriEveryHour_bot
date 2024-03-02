use std::sync::Arc;

use serenity::{
    all::{CommandInteraction, GuildId, ResolvedOption},
    builder::CreateCommand,
    client::Context,
};

use super::YuriState;

mod ping;
pub mod yuri;

pub trait YuriInteraction {
    fn register() -> CreateCommand<'static>;

    async fn run(
        context: &Context,
        interaction: &CommandInteraction,
        state: Arc<YuriState>,
        options: &[ResolvedOption],
    ) -> anyhow::Result<()>;
}

pub async fn register_interactions(guild_id: GuildId, context: &Context) {
    debug!("registering guild interactions");

    let interactions = &[ping::Interaction::register(), yuri::Interaction::register()];

    match guild_id.set_commands(&context.http, interactions).await {
        Ok(commands) => info!(
            "registered guild interactions: {registered_interactions}",
            registered_interactions = commands
                .iter()
                .map(|command| command.name.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        ),
        Err(error) => error!("an error occurred while registering guild interactions: {error:#?}"),
    }
}

pub async fn run_interactions(
    command_name: &str,
    context: &Context,
    interaction: &CommandInteraction,
    state: Arc<YuriState>,
    options: &[ResolvedOption<'_>],
) -> anyhow::Result<()> {
    match command_name {
        "ping" => ping::Interaction::run(context, interaction, state, options).await,
        "yuri" => yuri::Interaction::run(context, interaction, state, options).await,
        _ => Ok(()),
    }
}
