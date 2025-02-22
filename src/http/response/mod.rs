mod into_parts;
mod into_response;

use crate::StatusCode;

pub use into_response::IntoResponse;
use jiff::Zoned;

use super::header::HeaderMap;

#[derive(Debug, Clone, Default)]
pub struct Parts {
    pub status: StatusCode,
    pub headers: HeaderMap,
}

/// Represents an HTTP response.
#[derive(Debug, Clone, Default)]
pub struct Response {
    head: Parts,
    body: Option<Vec<u8>>,
}

impl Response {
    /// Creates a new response with the given body.
    ///
    /// ```
    /// use snx::response::Response;
    ///
    /// let res = Response::new("hello world!".as_bytes().to_vec());
    /// ```
    pub fn new(body: Vec<u8>) -> Self {
        Self {
            body: Some(body),
            ..Default::default()
        }
    }

    /// Gets a reference to the HTTP status code.
    ///
    /// ```
    /// use snx::{response::Response, StatusCode};
    ///
    /// let res = Response::new("hello world!".as_bytes().to_vec());
    /// let status = res.status();
    /// ```
    pub fn status(&self) -> &StatusCode {
        &self.head.status
    }

    /// Gets a mutable reference to the HTTP status code.
    ///
    /// ```
    /// use snx::{response::Response, StatusCode};
    ///
    /// let mut res = Response::new("hello world!".as_bytes().to_vec());
    /// *res.status_mut() = StatusCode::NotFound;
    /// ```
    pub fn status_mut(&mut self) -> &mut StatusCode {
        &mut self.head.status
    }

    /// Gets a reference to the HTTP headers.
    ///
    /// ```
    /// use snx::response::Response;
    ///
    /// let res = Response::new("hello world!".as_bytes().to_vec());
    /// let headers = res.headers();
    /// ```
    pub fn headers(&self) -> &HeaderMap {
        &self.head.headers
    }

    /// Gets a mutable reference to the HTTP headers.
    ///
    /// ```
    /// use snx::response::Response;
    ///
    /// let mut res = Response::new("hello world!".as_bytes().to_vec());
    /// *res.headers_mut() = ("Content-Type", "application/json").into();
    /// ```
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.head.headers
    }

    /// Gets a reference to the body.
    ///
    /// ```
    /// use snx::{response::Response, StatusCode};
    ///
    /// let res = Response::new("hello world!".as_bytes().to_vec());
    /// let body = res.body();
    /// ```
    pub fn body(&self) -> &Option<Vec<u8>> {
        &self.body
    }

    /// Serializes the response object to a raw HTTP response.
    ///
    /// ```
    /// use snx::response::Response;
    ///
    /// let bytes = Response::default().serialize_to_raw_http_response();
    /// ```
    pub fn serialize_to_raw_http_response(self) -> Vec<u8> {
        let mut serialized = Vec::new();

        serialized.extend_from_slice(
            format!(
                "HTTP/1.1 {} {}\r\n",
                self.head.status,
                self.head.status.canonical_reason()
            )
            .as_bytes(),
        );

        for (key, values) in self.head.headers.iter() {
            for value in values {
                serialized.extend_from_slice(format!("{}: {}\r\n", key, value).as_bytes());
            }
        }

        if let Some(ref body) = self.body {
            serialized.extend_from_slice(format!("Content-Length: {}\r\n", body.len()).as_bytes());
        }

        let date = Zoned::now()
            .strftime("Date: %a, %d %b %Y %H:%M:%S GMT\r\n")
            .to_string();
        serialized.extend_from_slice(date.as_bytes());

        serialized.extend_from_slice(b"\r\n");
        if let Some(body) = self.body {
            serialized.extend_from_slice(&body);
        }

        serialized
    }
}
