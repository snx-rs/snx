use crate::response::{IntoResponse, Response};

/// Represents an HTML response.
pub struct Html(pub String);

impl IntoResponse for Html {
    fn into_response(self) -> Response {
        let mut res = Response::new(self.0.as_bytes().to_vec());

        res.headers_mut()
            .insert("Content-Type", "text/html; charset=utf-8");

        res
    }
}
