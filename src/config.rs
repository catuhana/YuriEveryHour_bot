use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub mastodon: MastodonConfig,
    pub discord: DiscordConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MastodonConfig {
    #[serde(alias = "instance-host")]
    pub instance_host: String,
    #[serde(alias = "access-token")]
    pub access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub token: String,
    #[serde(alias = "server-id")]
    pub server_id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}
