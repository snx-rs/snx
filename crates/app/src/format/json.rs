use http::{
    HeaderMap, HeaderName, HeaderValue,
    into_response::{Body, IntoResponse},
};
use http_ext::Response;
use serde::{Serialize, de::DeserializeOwned};

use crate::format::Format;

pub struct Json<T>(pub T);

impl<T: DeserializeOwned> Format for Json<T> {
    type Value = T;
    type Error = serde_json::Error;

    fn from_bytes(bytes: &[u8]) -> Result<Self::Value, Self::Error> {
        serde_json::from_slice(bytes)
    }
}

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response<Body> {
        (
            HeaderMap::from_iter(vec![(
                HeaderName::from_static("content-type"),
                HeaderValue::from_static("application/json; charset=utf-8"),
            )]),
            serde_json::ser::to_string(&self.0).unwrap(),
        )
            .into_response()
    }
}
