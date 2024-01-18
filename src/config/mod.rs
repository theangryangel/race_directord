pub(crate) mod connection;
pub(crate) mod web;

use crate::Result;

use connection::Connection;
use toml::Table;
use web::Web;

use std::{collections::HashMap, fs, path::Path};

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq, PartialOrd)]
#[serde(tag = "plugin")]
pub(crate) enum PluginConfig {
    MyEvent { chance: f32 },
}

#[derive(Debug, Deserialize)]
pub(crate) struct Config {
    pub tick_rate: u64,

    pub web: Web,
    pub connection: Connection,
    pub plugins: HashMap<String, Table>,
}

impl Config {
    pub(crate) fn try_parse(file: &Path) -> Result<Self> {
        let contents = fs::read_to_string(file)?;
        let config: Self = toml::from_str(&contents)?;
        Ok(config)
    }
}
