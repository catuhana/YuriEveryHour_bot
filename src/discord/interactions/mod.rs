use serenity::{
    all::{CommandInteraction, GuildId, ResolvedOption},
    builder::CreateCommand,
    client::Context,
};

mod ping;

pub trait YuriInteraction {
    fn register() -> CreateCommand<'static>;

    async fn run(
        context: &Context,
        interaction: &CommandInteraction,
        options: &[ResolvedOption],
    ) -> anyhow::Result<()>;
}

pub async fn register_interactions(guild_id: GuildId, context: &Context) {
    let interactions = &[ping::PingInteraction::register()];

    if let Err(error) = guild_id.set_commands(&context.http, interactions).await {
        error!("an error ocurred while registering guild interactions: {error:#?}")
    } else {
        debug!("registered interactions");
    }
}

pub async fn run_interactions(
    command_name: &str,
    context: &Context,
    interaction: &CommandInteraction,
    options: &[ResolvedOption<'_>],
) -> anyhow::Result<()> {
    match command_name {
        "ping" => ping::PingInteraction::run(context, interaction, options).await,
        _ => Ok(()),
    }
}
