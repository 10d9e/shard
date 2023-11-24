use config::{Config, ConfigError};
use libp2p::Multiaddr;
use serde::{Serialize, Deserialize};
use tracing::debug;
use std::str::FromStr;
use std::{path::PathBuf, fs};
use std::fs::File;
use std::io::Write;
use std::io::Read;

#[derive(Debug, Serialize, Deserialize)]
pub struct ShardConfig {
    config_path: PathBuf,
    pub bootstrappers: Vec<Multiaddr>,
}

impl ShardConfig {
    pub fn new(path: &str) -> Result<Self, ConfigError> {
        let config_path = PathBuf::from(path);

        // create the config directory if it doesn't exist as a one-liner
        if !config_path.exists() {
            fs::create_dir_all(config_path.clone()).unwrap();
        }

        // only create a key if one doesn't exist
        if !config_path.join("key").exists() {
            let mut key_file = File::create(config_path.join("key")).unwrap();
            let rand_keys = libp2p::identity::Keypair::generate_ed25519();
            let encoded_priv = rand_keys.to_protobuf_encoding().unwrap();
            key_file.write_all(hex::encode(encoded_priv).as_bytes()).unwrap();
        }

        // if the conf.toml file doesn't exist, create it
        let config_path = config_path.canonicalize().unwrap();
        if !config_path.join("conf.toml").exists() {
            let shard_config = ShardConfig{
                config_path: config_path.clone(),
                bootstrappers: vec![],
            };
            let toml = toml::to_string_pretty(&shard_config).map_err(|err| ConfigError::Foreign(Box::new(err)))?;
            let config_source = config_path.to_str();
            let conf_file = config_source.unwrap().to_owned() + "/conf.toml";
            fs::write(conf_file, toml).unwrap();
        }

        debug!("📝 Loaded config at path: {:#?}", config_path);
        let config_source = config_path.to_str();
        let conf_file = config_source.unwrap().to_owned() + "/conf.toml";
        let settings = Config::builder()
            // Add in `./.shard/conf.toml`
            .add_source(config::File::with_name(&conf_file))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `SHARD_DEBUG=1 ./target/shard` would set the `debug` key
            .add_source(config::Environment::with_prefix("SHARD"))
            .build()
            .unwrap();

        let my_config: ShardConfig = settings.try_into()?;
        Ok(my_config)
    }

    pub fn key(&self) -> libp2p::identity::Keypair {
        let mut key_file = File::open(self.config_path.join("key")).unwrap();
        let mut encoded_priv = String::new();
        key_file.read_to_string(&mut encoded_priv).unwrap();
        let out = hex::decode(encoded_priv).unwrap();
        libp2p::identity::Keypair::from_protobuf_encoding(&out).unwrap()
    }

    pub fn peer_id(&self) -> libp2p::PeerId {
        self.key().public().to_peer_id()
    }

}

impl TryFrom<Config> for ShardConfig {
    type Error = ConfigError;

    fn try_from(config: Config) -> Result<Self, Self::Error> {
        let bootstrappers: Vec<libp2p::Multiaddr> = config.get_array("bootstrappers")?
            .into_iter()
            .map(|v| Multiaddr::from_str(&v.into_string().unwrap()).unwrap()) 
            .collect();
        Ok(
            ShardConfig {
                bootstrappers,
                config_path: config.get_string("config_path")?.into(),
            }
        )
    }
}
