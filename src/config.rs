use std::{fs::File, io::Read};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub server: ServerConfig,
}

#[derive(Deserialize, Debug)]
pub struct ServerConfig {
    pub base_url: String,
    pub host: String,
    pub port: u16,
    pub num_threads: usize,
}

impl Config {
    /// Tries to read and parse the config from the filesystem.
    ///
    /// ```no_run
    /// use snx::Config;
    ///
    /// let config = Config::try_from_fs().unwrap();
    /// ```
    pub fn try_from_fs() -> anyhow::Result<Self> {
        let mut contents = String::new();
        File::open("./snx.toml")?.read_to_string(&mut contents)?;

        Ok(toml::from_str::<Config>(&contents)?)
    }
}
