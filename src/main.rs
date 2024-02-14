use clap::Parser;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use figment_file_provider_adapter::FileAdapter;
use tracing_subscriber::EnvFilter;

mod cli;
mod config;
mod discord;

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
            // TODO: Some things does not work as intended, probably related to library
            // and created an issue for it already. Check and remove this comment after
            // fix.
            let config = Figment::new()
                .merge(FileAdapter::wrap(Yaml::file(config)).with_suffix("-file"))
                .merge(FileAdapter::wrap(Env::raw()))
                .extract::<config::Config>()?;

            discord::YuriDiscord::from(config.discord).spawn().await?;
        }
    }

    Ok(())
}
