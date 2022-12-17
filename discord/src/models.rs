use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub discord: Discord,
    pub deadname: String,
}

#[derive(Deserialize, Debug)]
pub struct Discord {
    pub token: String,
    pub target_channels: Vec<u64>,
}
