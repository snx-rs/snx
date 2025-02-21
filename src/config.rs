use std::{fs::File, io::Read};

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
    pub database: DatabaseConfig,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub base_url: String,
    pub host: String,
    pub port: u16,
    pub num_threads: usize,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DatabaseConfig {
    /// A database connection URL used to connect to the database.
    pub url: String,
    /// Minimum idle connection count maintained by the pool. Shouldn't be greater than `max_size`.
    ///
    /// Defaults to None.
    pub min_idle: Option<u32>,
    /// Maximum number of connections managed by the database connection pool.
    ///
    /// Defaults to 10.
    pub max_size: Option<u32>,
    /// Maximum lifetime of connections in the database connection pool in seconds.
    ///
    /// Defaults to 30 minutes.
    pub max_lifetime: Option<u64>,
    /// Idle timeout used by the database connection pool in seconds.
    ///
    /// Defaults to 10 minutes.
    pub idle_timeout: Option<u64>,
    /// Connection timeout used by the pool in seconds.
    ///
    /// Defaults to 30 seconds.
    pub connection_timeout: Option<u64>,
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
