use std::{collections::HashMap, fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

/// Manages configuration variables
/// All configuration details are specified in `velocity.toml`
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// your key to the instatus api
    pub api_key: String,
    /// endpoints to monitor
    pub endpoints: HashMap<String, String>,
    /// frequency to monitor endpoints, in seconds
    /// your endpoints will be pinged every `frequency` seconds
    pub frequency: u64,
    /// maximum connection timeout for all endpoints
    /// default: 10s
    pub max_connection_timeout: Option<u8>,
}

impl Config {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Self {
        let mut file = File::open(path).unwrap_or_else(|error| {
            eprintln!("💥 failed to read config file: {}", error);
            std::process::exit(1);
        });

        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        let mut config = serde_json::from_str::<Config>(&contents).unwrap();

        if config.max_connection_timeout.is_none() {
            config.max_connection_timeout = Some(10);
        }

        config
    }
}
