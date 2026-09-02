use std::{
    io, iter,
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use futures::{Stream, stream};
use http_ext::{Response, StatusCode, response::Parts};

use crate::into_parts::IntoParts;

/// The response body is always a stream of bytes. Non-stream responses are converted into one-shot
/// streams.
pub struct Body(Pin<Box<dyn Stream<Item = io::Result<Bytes>> + Send>>);

/// Represents everything that can be turned into the body of a response.
pub trait IntoBody {
    fn into_body(self) -> Body;
}

impl<S: Stream<Item = io::Result<Bytes>> + Send + 'static> IntoBody for S {
    fn into_body(self) -> Body {
        Body(Box::pin(self))
    }
}

impl Stream for Body {
    type Item = io::Result<Bytes>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0.as_mut().poll_next(cx)
    }
}

/// Represents everything that can be turned into a response.
///
/// Only types that implement [IntoResponse] can be returned from handlers.
pub trait IntoResponse: IntoResponseBoxed {
    fn into_response(self) -> Response<Body>;
}

impl IntoResponse for () {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .body(stream::empty::<io::Result<Bytes>>().into_body())
            .unwrap()
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(self)
            .body(stream::empty::<io::Result<Bytes>>().into_body())
            .unwrap()
    }
}

impl IntoResponse for i32 {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .status(StatusCode::from_u16(self.try_into().unwrap()).unwrap())
            .body(stream::empty::<io::Result<Bytes>>().into_body())
            .unwrap()
    }
}

impl IntoResponse for &str {
    fn into_response(self) -> Response<Body> {
        Response::builder()
            .header("Content-Type", "text/plain; charset=utf-8")
            .header("Content-Length", self.len())
            .body(one_shot(Bytes::copy_from_slice(self.as_bytes())))
            .unwrap()
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response<Body> {
        self.as_str().into_response()
    }
}

impl<R> IntoResponse for (Parts, R)
where
    R: IntoResponse,
{
    fn into_response(self) -> Response<Body> {
        let mut res = self.1.into_response();

        *res.status_mut() = self.0.status;
        *res.version_mut() = self.0.version;
        *res.headers_mut() = self.0.headers;
        *res.extensions_mut() = self.0.extensions;

        res
    }
}

macro_rules! define_into_response_for_tuple {
    ($($t:ident),*) => {
        #[allow(non_snake_case)]
        impl<$($t,)* R> IntoResponse for ($($t,)* R)
        where
            $($t: IntoParts + Clone,)*
            R: IntoResponse
        {
            fn into_response(self) -> Response<Body> {
                let ($($t,)* res) = self;

                let (mut parts, _) = Response::new(()).into_parts();
                $(
                    parts = $t.clone().into_parts(parts.clone());
                )*

                (parts, res).into_response()
            }
        }
    };
}

define_into_response_for_tuple!(T1);
define_into_response_for_tuple!(T1, T2);
define_into_response_for_tuple!(T1, T2, T3);
define_into_response_for_tuple!(T1, T2, T3, T4);
define_into_response_for_tuple!(T1, T2, T3, T4, T5);
define_into_response_for_tuple!(T1, T2, T3, T4, T5, T6);
define_into_response_for_tuple!(T1, T2, T3, T4, T5, T6, T7);
define_into_response_for_tuple!(T1, T2, T3, T4, T5, T6, T7, T8);

fn one_shot(bytes: Bytes) -> Body {
    stream::iter(iter::once(Ok(bytes))).into_body()
}

/// Used to allow IntoResponse dynamic trait objects to be converted into a response using
/// .into_response(self) without forcing functions with static dispatch which return an impl
/// IntoResponse to box their return values.
///
/// Seems that the only options to allow impl IntoResponse's and Box<dyn IntoResponse>'s to both be
/// transformed into a response are:
///     1. Change the receiver from self to Box<Self>, but this forces handlers to box their return
///        values
///     2. Implement a second method `into_response_boxed(self: Box<Self>)`, but this forces
///        everyone that wants to implement IntoResponse themselves to also implement this
///        "redundant" method
///     3. Change the receiver to `into_response(&self)`, but this won't consume self, and thus
///        shouldn't be called "Into*"
///     4. Use this trickery below to magically allow `.into_response(self)` on Box<dyn
///        IntoResponse>'s
pub trait IntoResponseBoxed {
    fn into_response_boxed(self: Box<Self>) -> Response<Body>;
}

impl<T> IntoResponseBoxed for T
where
    T: IntoResponse,
{
    fn into_response_boxed(self: Box<Self>) -> Response<Body> {
        (*self).into_response()
    }
}

impl<T> IntoResponse for Box<T>
where
    T: ?Sized + IntoResponseBoxed,
{
    fn into_response(self) -> Response<Body> {
        self.into_response_boxed()
    }
}
