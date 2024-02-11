use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub subcommand: SubCommands,
}

#[derive(Subcommand, Debug, Serialize, Deserialize)]
pub enum SubCommands {
    /// Start the bot
    Start {
        /// Path to the configuration file
        #[arg(env, long, default_value_t = dirs::config_dir()
            .expect("unsupported operating system or platform")
            .join("YuriEveryDay_bot")
            .join("config")
            .with_extension("yaml")
            .as_path()
            .display()
            .to_string()
        )]
        config: String,
    },
}
