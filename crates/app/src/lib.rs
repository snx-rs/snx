pub mod builder;
pub mod format;
pub mod handler;
pub mod request;
pub mod router;
pub mod serve;
pub mod validation;

use std::{io, sync::Arc};

use crate::builder::Builder;

/// Holds the definition of your application
///
/// ```
/// App::builder()
///     .with_routes(|builder| {
///         builder
///             .get("/", handlers::index)
///             .get("/users", handlers::users::index)
///     })
///     .build()?
///     .boot(snx::tokio::runtime)
/// ```
pub struct App {
    router: router::Router,
}

impl App {
    /// Creates a new builder instance
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Boots this application on the given runtime
    pub fn boot<R>(self, rt: R) -> io::Result<()>
    where
        R: Fn(Arc<App>) -> io::Result<()>,
    {
        rt(Arc::new(self))
    }
}
