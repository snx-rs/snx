use std::time::Duration;

use crate::Config;

#[derive(Clone)]
pub struct Context {
    pub config: Config,
    #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
    pub db: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<crate::db::DatabaseConnection>>,
}

impl Context {
    pub fn new(config: Config) -> Self {
        Self {
            config: config.clone(),
            #[cfg(any(feature = "sqlite", feature = "postgres", feature = "mysql"))]
            db: {
                let mut pool = diesel::r2d2::Pool::builder()
                    .test_on_check_out(true)
                    .min_idle(config.database.min_idle)
                    .idle_timeout(config.database.idle_timeout.map(Duration::from_secs))
                    .max_lifetime(config.database.max_lifetime.map(Duration::from_secs));

                if let Some(connection_timeout) = config.database.connection_timeout {
                    pool = pool.connection_timeout(Duration::from_secs(connection_timeout));
                }

                if let Some(max_size) = config.database.max_size {
                    pool = pool.max_size(max_size);
                }

                let manager = diesel::r2d2::ConnectionManager::<crate::db::DatabaseConnection>::new(
                    config.database.url,
                );

                pool.build(manager).unwrap()
            },
        }
    }
}
