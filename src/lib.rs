use http::router::Router;
use rayon::ThreadPoolBuilder;
use serde::Deserialize;

mod http;

pub use http::router;
pub use http::{Request, Response};

/// Describes an snx application with sane defaults.
pub trait App {
    /// Defines the application's routes.
    fn with_routes() -> Router;

    /// Defines the application's configuration.
    ///
    /// Reads the .env file at the root by default.
    fn with_config() -> anyhow::Result<Config> {
        dotenvy::dotenv()?;

        Ok(envy::from_env::<Config>()?)
    }

    /// Defines the application's logging functionality.
    ///
    /// Sets up line-oriented logging to stdout by default.
    fn with_logging() {
        tracing_subscriber::fmt::init();
    }
}

/// Boots the snx framework and starts serving your application.
pub fn boot<A: App>() -> anyhow::Result<()> {
    let config = A::with_config()?;
    let addr = format!("{}:{}", config.host, config.port);
    let pool = ThreadPoolBuilder::new()
        .num_threads(config.num_threads)
        .build()?;

    let router = A::with_routes();

    A::with_logging();

    Ok(http::serve(addr, pool, router)?)
}

/// The configuration for your application.
///
/// This is read at runtime from the environment variables (and .env file).
#[derive(Deserialize)]
pub struct Config {
    host: String,
    port: usize,
    num_threads: usize,
}
