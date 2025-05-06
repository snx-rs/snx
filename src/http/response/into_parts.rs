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

#[cfg(feature = "cookies")]
impl IntoParts for biscotti::ResponseCookies<'_> {
    fn into_parts(self, parts: Parts) -> Parts {
        let mut parts = parts.clone();

        let processor: biscotti::Processor = biscotti::ProcessorConfig::default().into();
        parts.headers.insert(
            "set-cookie",
            &self
                .header_values(&processor)
                .collect::<Vec<String>>()
                .join("; "),
        );

        parts
    }
}
