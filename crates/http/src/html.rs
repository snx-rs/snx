use std::fmt::Display;

use http_ext::{HeaderMap, HeaderName, HeaderValue};

use crate::into_response::IntoResponse;

pub struct Html<T: Display>(pub T);

impl<T: Display> IntoResponse for Html<T> {
    fn into_response(self) -> http_ext::Response<crate::into_response::Body> {
        (
            HeaderMap::from_iter(vec![(
                HeaderName::from_static("content-type"),
                HeaderValue::from_static("text/html; charset=utf-8"),
            )]),
            self.0.to_string(),
        )
            .into_response()
    }
}
