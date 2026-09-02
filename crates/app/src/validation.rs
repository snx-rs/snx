use http::into_response::{Body, IntoResponse};
use http_ext::Response;

pub trait Validatable: Sized {
    fn validate(self) -> Result<Self, Error>;
}

#[derive(Debug)]
pub struct Error;

impl IntoResponse for Error {
    fn into_response(self) -> Response<Body> {
        todo!()
    }
}

pub trait Rule<T: Sized> {
    fn validate(&self, value: T) -> Result<(), Error>;
}

macro_rules! rule {
    ($name:ident, |$value:ident: $ty:ty $(, $field:ident: $fty:ty)*| $body:expr) => {
        pub struct $name {
            $(pub $field: $fty),*
        }

        impl Rule<$ty> for $name {
            fn validate(&self, $value: $ty) -> Result<(), Error> {
                let Self { $($field),* } = self;
                if $body { Ok(()) } else {Err(Error)}
            }
        }
    };
}

#[rustfmt::skip]
pub mod rules {
    use std::ops::Range;

    use crate::validation::{Rule, Error};

    rule!(Ascii, |value: &str| value.is_ascii());
    rule!(Alpha, |value: &str| value.chars().all(|c| c.is_alphabetic()));
    rule!(Alphanumeric, |value: &str| value.chars().all(|c| c.is_alphanumeric()));
    rule!(Length, |value: &str, range: Range<usize>| range.contains(&value.len()));
}

impl Validatable for () {
    fn validate(self) -> Result<Self, Error> {
        Ok(())
    }
}
