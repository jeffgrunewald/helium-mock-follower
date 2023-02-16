use config::{Config, Environment, File};
use serde::Deserialize;
use std::{
    net::{AddrParseError, SocketAddr},
    path::Path,
    str::FromStr,
};

#[derive(Debug, Deserialize)]
pub struct Settings {
    /// RUST_LOG compatible settings string. Default to
    /// "helium_mock_follower=info"
    #[serde(default = "default_log")]
    pub log: String,
    /// Listen address. Required. Default is 0.0.0.0:8080
    #[serde(default = "default_listen_addr")]
    pub listen: String,
    /// Starting block height
    #[serde(default = "default_height")]
    pub height: u64,
    /// File path of gateways to load at runtime
    pub gateways: Option<String>,
}

pub fn default_log() -> String {
    "helium_mock_follower=debug".to_string()
}

pub fn default_listen_addr() -> String {
    "0.0.0.0:8080".to_string()
}

pub fn default_height() -> u64 {
    1700000
}

impl Settings {
    /// Settings can be loaded from a given optional path and
    /// can be overridden with environment variables.
    ///
    /// Environment overrides have the same name as the entries
    /// in the settings file in uppercase and prefixed with "FLW_".
    /// Example: "FLW_LISTEN" will override the grpc listen address.
    pub fn new<P: AsRef<Path>>(path: Option<P>) -> Result<Self, config::ConfigError> {
        let mut builder = Config::builder();

        // Maybe add optional config file
        if let Some(file) = path {
            builder = builder
                .add_source(File::with_name(&file.as_ref().to_string_lossy()).required(false));
        }

        // Maybe add any environment variables
        builder
            .add_source(Environment::with_prefix("FLW").separator("_"))
            .build()
            .and_then(|config| config.try_deserialize())
    }

    pub fn listen_addr(&self) -> Result<SocketAddr, AddrParseError> {
        SocketAddr::from_str(&self.listen)
    }
}
