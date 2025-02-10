use crate::{http::header::HeaderMap, response::into_parts::IntoParts, StatusCode};

use super::{Parts, Response};

/// Represents everything that can be turned into a response.
///
/// Types that implement [IntoResponse] can be returned from handlers.
pub trait IntoResponse: IntoResponseBoxed {
    fn into_response(self) -> Response;
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::default()
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        let mut res = Response::default();

        *res.status_mut() = self;

        res
    }
}

impl IntoResponse for &str {
    fn into_response(self) -> Response {
        let mut res = Response::new(self.as_bytes().to_vec());

        res.headers_mut()
            .insert("Content-Type", "text/plain; charset=utf-8");

        res
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        let mut res = Response::new(self.into_bytes());

        res.headers_mut()
            .insert("Content-Type", "text/plain; charset=utf-8");

        res
    }
}

impl IntoResponse for HeaderMap {
    fn into_response(self) -> Response {
        let mut res = Response::default();

        *res.headers_mut() = self;

        res
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl<R> IntoResponse for (Parts, R)
where
    R: IntoResponse,
{
    fn into_response(self) -> Response {
        let mut res = self.1.into_response();

        *res.status_mut() = self.0.status;
        *res.headers_mut() = self.0.headers;

        res
    }
}

impl<T, E> IntoResponse for Result<T, E>
where
    T: IntoResponse,
    E: IntoResponse,
{
    fn into_response(self) -> Response {
        match self {
            Ok(t) => t.into_response(),
            Err(e) => e.into_response(),
        }
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
            fn into_response(self) -> Response {
                let ($($t,)* res) = self;

                let mut parts = Parts::default();
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
    fn into_response_boxed(self: Box<Self>) -> Response;
}

impl<T> IntoResponseBoxed for T
where
    T: IntoResponse,
{
    fn into_response_boxed(self: Box<Self>) -> Response {
        (*self).into_response()
    }
}

impl<T> IntoResponse for Box<T>
where
    T: ?Sized + IntoResponse,
{
    fn into_response(self) -> Response {
        self.into_response_boxed()
    }
}
