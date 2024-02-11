use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub mastodon: MastodonConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MastodonConfig {
    #[serde(rename = "instance-host")]
    pub instance_host: String,
    #[serde(rename = "access-token")]
    pub access_token: String,
}
