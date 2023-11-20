use config::{Config, ConfigError};
use libp2p::Multiaddr;
use serde::{Serialize, Deserialize};
use tracing::debug;
use std::{path::PathBuf, fs};

#[derive(Debug, Serialize, Deserialize)]
pub struct ShardConfig {
    pub bootstrapper: Option<Multiaddr>,
}

impl ShardConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let config_path = PathBuf::from(".shard/conf.toml");

        if !config_path.exists() {
            if let Some(dir) = config_path.parent() {
                if !dir.exists() {
                    fs::create_dir_all(dir).unwrap();
                }
            }
    
            let toml = toml::to_string_pretty(&ShardConfig::default()).map_err(|err| ConfigError::Foreign(Box::new(err)))?;
            fs::write(&config_path, toml).unwrap();
        }

        debug!("📝 Loaded config at path: {:?}", config_path);

        let settings = Config::builder()
            // Add in `./.shard/conf.toml`
            .add_source(config::File::with_name(".shard/conf"))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `SHARD_DEBUG=1 ./target/shard` would set the `debug` key
            .add_source(config::Environment::with_prefix("SHARD"))
            .build()
            .unwrap();

        let my_config: ShardConfig = settings.try_into()?;
        Ok(my_config)
    }

    fn default() -> Self {
        ShardConfig {
            bootstrapper: Some("/ip4/127.0.0.1/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X".parse().unwrap()),
        }
    }
}

impl TryFrom<Config> for ShardConfig {
    type Error = ConfigError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        Ok(
            ShardConfig {
                bootstrapper: Some(config.get_string("bootstrapper")?.parse().unwrap()),
            }
        )
    }
}
