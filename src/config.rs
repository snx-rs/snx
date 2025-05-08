use std::{fs::File, io::Read, num::ParseIntError, str::FromStr, time::Duration};

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
    pub database: DatabaseConfig,
    #[cfg(feature = "sessions")]
    pub session: Option<SessionConfig>,
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

#[derive(Deserialize, Debug, Clone, Default)]
pub struct SessionConfig {
    /// Cookie key used for storing the session.
    ///
    /// Defaults to 'snx-session'.
    pub cookie_key: Option<String>,
    /// Duration after which the session will expire.
    ///
    /// Defaults to 7 days.
    pub expires_after: Option<String>,
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

/// Represents an error that occurred during duration parsing.
#[derive(thiserror::Error, Debug)]
pub enum ParseDurationError {
    #[error("invalid format")]
    InvalidFormat,
    #[error(transparent)]
    InvalidNumber(#[from] ParseIntError),
}

/// Parses a duration string into a Duration struct.
///
/// Examples: "30m" for 30 minutes and "7d" for 7 days.
pub fn parse_duration(value: &str) -> Result<Duration, ParseDurationError> {
    if value.len() < 2 {
        return Err(ParseDurationError::InvalidFormat);
    }

    let (value, unit) = value.split_at(value.len() - 1);
    let value = u64::from_str(value)?;

    match unit {
        "m" => Ok(Duration::from_secs(value * 60)),
        "d" => Ok(Duration::from_secs(value * 24 * 60 * 60)),
        _ => Err(ParseDurationError::InvalidFormat),
    }
}
