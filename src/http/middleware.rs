use std::{sync::Arc, time::SystemTime};

use super::{
    request::Request,
    response::{IntoResponse, Response},
};

pub type MiddlewareHandler =
    Arc<Box<dyn Fn(Request, Box<dyn Fn() -> Response>) -> Box<dyn IntoResponse> + Send + Sync>>;

/// Built-in middleware to trace requests.
pub fn trace_requests(req: Request, next: Box<dyn Fn() -> Response>) -> Box<dyn IntoResponse> {
    let now = SystemTime::now();

    let res = next();

    let elapsed = now.elapsed().unwrap();

    tracing::info!(
        "{} \"{} {}\" {} {}B {}ms",
        req.peer_addr().map(|p| p.to_string()).unwrap_or_default(),
        req.method(),
        req.path(),
        res.status(),
        res.body().clone().unwrap_or_default().len(),
        elapsed.as_millis(),
    );

    Box::new(res)
}
