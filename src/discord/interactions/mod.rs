use serenity::{
    all::{CommandInteraction, GuildId, ResolvedOption},
    builder::CreateCommand,
    client::Context,
};
use sqlx::PgPool;

mod ping;
mod yuri;

pub trait YuriInteraction {
    fn register() -> CreateCommand<'static>;

    async fn run(
        context: &Context,
        database: PgPool,
        interaction: &CommandInteraction,
        options: &[ResolvedOption],
    ) -> anyhow::Result<()>;
}

pub async fn register_interactions(guild_id: GuildId, context: &Context) {
    let interactions = &[
        ping::PingInteraction::register(),
        yuri::YuriCInteraction::register(),
    ];

    if let Err(error) = guild_id.set_commands(&context.http, interactions).await {
        error!("an error ocurred while registering guild interactions: {error:#?}")
    } else {
        debug!("registered interactions");
    }
}

pub async fn run_interactions(
    command_name: &str,
    context: &Context,
    database: PgPool,
    interaction: &CommandInteraction,
    options: &[ResolvedOption<'_>],
) -> anyhow::Result<()> {
    match command_name {
        "ping" => ping::PingInteraction::run(context, database, interaction, options).await,
        "yuri" => yuri::YuriCInteraction::run(context, database, interaction, options).await,
        _ => Ok(()),
    }
}
