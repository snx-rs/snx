use crate::response::{IntoResponse, Response};

/// Represents an HTML response.
pub struct Html<'a>(pub &'a str);

impl IntoResponse for Html<'_> {
    fn into_response(self) -> Response {
        let mut res = Response::new(self.0.as_bytes().to_vec());

        res.headers_mut()
            .insert("Content-Type", "text/html; charset=utf-8");

        res
    }
}
