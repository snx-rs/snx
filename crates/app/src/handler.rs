use std::{marker::PhantomData, pin::Pin, sync::Arc};

use http::into_response::IntoResponse;

use crate::{format::Format, request::Request, validation::Validatable};

/// Represents a request handler
///
/// A request handler processes a request and turns it into something that can be turned into a
/// response.
pub trait Handler: Send + Sync {
    fn call(&self, request: Request)
    -> Pin<Box<dyn Future<Output = Box<dyn IntoResponse>> + Send>>;
}

pub struct HandlerFn<Func, F> {
    func: Func,
    format: PhantomData<F>,
}

impl<Func, F> HandlerFn<Func, F> {
    pub fn new(func: Func) -> Self {
        Self {
            func,
            format: PhantomData,
        }
    }
}

impl<Func, F, Fut, R> Handler for HandlerFn<Func, F>
where
    Func: Fn(Request<F>) -> Fut + Send + Sync,
    Fut: Future<Output = R> + Send + 'static,
    F: Format + Send + Sync,
    F::Value: Validatable,
    R: IntoResponse + 'static,
{
    fn call(
        &self,
        request: Request,
    ) -> Pin<Box<dyn Future<Output = Box<dyn IntoResponse>> + Send>> {
        let fut = (self.func)(request.retype::<F>());
        Box::pin(async move { Box::new(fut.await) as Box<dyn IntoResponse> })
    }
}

impl Handler for () {
    fn call(
        &self,
        _: Request,
    ) -> Pin<Box<dyn Future<Output = Box<dyn IntoResponse + 'static>> + Send + 'static>> {
        Box::pin(async move { Box::new(()) as Box<dyn IntoResponse> })
    }
}

/// Executes the given handler and passes on the given request
pub async fn trigger(
    handler: Arc<Box<dyn Handler + Send + Sync>>,
    request: Request,
) -> Box<dyn IntoResponse> {
    handler.call(request).await
}
