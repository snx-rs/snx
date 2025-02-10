use crate::{http::header::HeaderMap, StatusCode};

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

impl IntoParts for HeaderMap {
    fn into_parts(self, parts: Parts) -> Parts {
        let mut parts = parts.clone();

        parts.headers = self;

        parts
    }
}

impl<T> IntoParts for T
where
    T: AsRef<[(&'static str, &'static str)]>,
{
    fn into_parts(self, parts: Parts) -> Parts {
        let mut parts = parts.clone();

        for header in self.as_ref() {
            parts.headers.insert(header.0, header.1);
        }

        parts
    }
}
