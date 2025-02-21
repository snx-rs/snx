use std::sync::Arc;

use crate::Context;

use super::{request::Request, response::IntoResponse};

/// Represents a handler which processes a request and turns it into something that can be turned
/// into a response.
pub trait Handler: Send + Sync {
    fn call(&self, ctx: Context, request: Request) -> Box<dyn IntoResponse>;
}

impl<F, R> Handler for F
where
    F: Fn(Context, Request) -> R + Send + Sync,
    R: IntoResponse + 'static,
{
    fn call(&self, ctx: Context, request: Request) -> Box<dyn IntoResponse> {
        Box::new((self)(ctx, request))
    }
}

/// Executs the given handler and passes it the given request.
pub fn trigger(
    ctx: Context,
    request: Request,
    handler: Arc<Box<dyn Handler + Send + Sync>>,
) -> Box<dyn IntoResponse> {
    handler.call(ctx, request)
}
