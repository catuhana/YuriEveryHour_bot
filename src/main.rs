use clap::Parser;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use figment_file_provider_adapter::FileAdapter;
use sqlx::PgPool;
use tracing_subscriber::EnvFilter;

mod cli;
mod config;
mod discord;
mod models;

#[macro_use]
extern crate tracing;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("yuri_every_hour_bot=info")),
        )
        .init();

    match cli::Cli::parse().subcommand {
        cli::SubCommands::Start { config } => {
            let config = Figment::new()
                .merge(FileAdapter::wrap(Yaml::file(config)).with_suffix("-file"))
                .merge(FileAdapter::wrap(Env::raw().split("__")))
                .extract::<config::Config>()?;

            let postgres_pool = PgPool::connect(&config.database.url).await?;
            sqlx::migrate!().run(&postgres_pool).await?;

            discord::YuriDiscord::new(config.discord, postgres_pool)
                .spawn()
                .await?;
        }
    }

    Ok(())
}
