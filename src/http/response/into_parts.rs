use crate::{http::header::Header, StatusCode};

use super::Parts;

/// Represents parts of the head of a response.
pub trait IntoParts {
    fn into_parts(self, parts: Parts) -> Parts;
}

impl IntoParts for StatusCode {
    fn into_parts(self, parts: Parts) -> Parts {
        let mut parts = parts.clone();

        parts.status = self;

        parts
    }
}

impl IntoParts for Vec<Header> {
    fn into_parts(self, parts: Parts) -> Parts {
        let mut parts = parts.clone();

        parts.headers = self;

        parts
    }
}
