use anyhow::Context;
use figment::Figment;
use figment::providers::Env;
use secp256k1::SecretKey;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct MainNet {
    pub deploy_service_url: String,
    pub propose_service_url: String,
    pub validator_ws_api_url: String,
    pub observer_url: String,
    pub observer_ws_api_url: String,
    pub service_key: SecretKey,
    pub wallets_env_key: SecretKey,
    pub agents_env_key: SecretKey,
    pub agents_teams_env_key: SecretKey,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestNet {
    pub deploy_service_url: String,
    pub propose_service_url: String,
    pub validator_ws_api_url: String,
    pub observer_url: String,
    pub observer_ws_api_url: String,
    pub service_key: SecretKey,
    pub env_key: SecretKey,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub address: String,
    pub port: u16,
    pub log_level: String,
    pub mainnet: MainNet,
    pub testnet: TestNet,
    #[serde(deserialize_with = "deserialize_hex_key")]
    pub aes_encryption_key: [u8; 32],
}

pub fn collect_config() -> anyhow::Result<Config> {
    Figment::new()
        .merge(Env::prefixed("EMBERS__").split("__"))
        .extract()
        .context("failed to collect config")
}

fn deserialize_hex_key<'de, D, const S: usize>(deserializer: D) -> Result<[u8; S], D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let mut array = [0u8; S];
    hex::decode_to_slice(&s, &mut array).map_err(serde::de::Error::custom)?;
    Ok(array)
}
