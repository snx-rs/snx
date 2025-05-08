use std::sync::{Arc, Mutex};

use crate::{
    config::Config,
    http::router::Router,
    middleware::{trace_requests, MiddlewareHandler},
    panic_hook::panic_hook,
    router, Context, Server,
};

/// Describes an snx application with sane defaults.
pub trait App {
    /// Defines the application's routes.
    ///
    /// Sets up an empty router by default.
    fn with_routes(builder: router::Builder) -> Router {
        builder.build().unwrap()
    }

    /// Defines the application's configuration.
    ///
    /// Reads the `snx.toml` at the project root by default.
    fn with_config() -> Config {
        Config::try_from_fs().expect("failed to retrieve config from file `snx.toml`")
    }

    /// Defines the application's global middleware.
    ///
    /// Configures the request tracing middleware by default.
    fn with_global_middleware() -> Vec<MiddlewareHandler> {
        vec![
            Arc::new(Box::new(trace_requests)),
            #[cfg(feature = "sessions")]
            Arc::new(Box::new(crate::middleware::initialize_session)),
        ]
    }

    /// Defines the application's logging/tracing configuration.
    ///
    /// Sets up a global tracing subcriber with line-oriented logging to stdout by default.
    fn with_tracing() {
        tracing_subscriber::fmt().with_target(false).init();
    }

    /// Defines the application's session store.
    #[cfg(feature = "sessions")]
    fn with_sessions(_: Context) -> Option<Box<dyn crate::session::SessionStore + Send + Sync>> {
        Some(Box::new(crate::session::MemorySessionStore::default()))
    }
}

/// Boots the snx framework and starts your application.
pub fn boot<A: App>() {
    let config = A::with_config();

    let mut ctx = Context::new(config.clone());

    #[cfg(feature = "sessions")]
    {
        ctx.session_store = A::with_sessions(ctx.clone()).map(|v| Arc::new(Mutex::new(v)));
    }

    let builder = Router::builder(&config.server.base_url);
    let router = A::with_routes(builder);
    let global_middleware = A::with_global_middleware();

    A::with_tracing();
    std::panic::set_hook(Box::new(panic_hook));

    let addr = format!("{}:{}", config.server.host, config.server.port);
    Server::try_bind(addr, router, ctx, global_middleware)
        .unwrap()
        .num_threads(config.server.num_threads)
        .serve();
}
