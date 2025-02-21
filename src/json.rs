use serde::Serialize;

use crate::response::{IntoResponse, Response};

/// Represents a JSON response.
pub struct Json<T: Serialize>(pub T);

impl<T: Serialize> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        let bytes = serde_json::to_vec(&self.0).expect("failed to serialize type into json bytes");

        let mut res = Response::new(bytes);

        res.headers_mut().insert("Content-Type", "application/json");

        res
    }
}

#[derive(Debug, Clone)]
pub struct InvalidJsonBodyError {
    message: String,
}

impl std::error::Error for InvalidJsonBodyError {}

impl std::fmt::Display for InvalidJsonBodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<serde_json::error::Error> for InvalidJsonBodyError {
    fn from(value: serde_json::error::Error) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}
