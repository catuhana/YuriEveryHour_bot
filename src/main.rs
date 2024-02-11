use clap::Parser;
use figment::{
    providers::{Env, Format, Yaml},
    Figment,
};
use figment_file_provider_adapter::FileAdapter;

mod cli;
mod config;
fn main() {
    match cli::Cli::parse().subcommand {
        cli::SubCommands::Start { config } => {
            // TODO: Some things does not work as intended, probably related to library
            // and created an issue for it already. Check and remove this comment after
            // fix.
            let config = Figment::new()
                .merge(FileAdapter::wrap(Yaml::file(config)).with_suffix("-file"))
                .merge(FileAdapter::wrap(Env::raw()))
                .extract::<config::Config>();

            match config {
                Ok(config) => {
                    dbg!(config);
                }
                Err(error) => {
                    eprintln!("Error: {error}")
                }
            }
        }
    }
}
