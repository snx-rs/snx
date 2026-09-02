use http_ext::{HeaderMap, StatusCode, response::Parts};

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
