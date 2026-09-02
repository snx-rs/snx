pub mod html;
pub mod into_parts;
pub mod into_response;

use std::str::FromStr;

pub use http_ext::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use http_ext::{Method, Uri, Version, request::Builder, response::Parts};
use httparse::{EMPTY_HEADER, ParserConfig, Request};
use jiff::Zoned;

const HEADERS_COUNT: usize = 32;

#[derive(thiserror::Error, Debug)]
pub enum ParseRequestError {
    #[error(transparent)]
    InvalidByte(#[from] httparse::Error),
    #[error(transparent)]
    InvalidMethod(#[from] http_ext::method::InvalidMethod),
    #[error(transparent)]
    InvalidUri(#[from] http_ext::uri::InvalidUri),
}

/// Parses the given buffer into an `http::request::Builder`
pub fn parse_request(buf: &[u8]) -> Result<Option<Builder>, ParseRequestError> {
    let mut headers = [EMPTY_HEADER; HEADERS_COUNT];
    let mut request = Request::new(&mut headers);

    match ParserConfig::default().parse_request(&mut request, buf) {
        Ok(httparse::Status::Complete(_)) => {
            let mut builder = http_ext::Request::builder()
                .method(Method::from_bytes(request.method.unwrap().as_bytes())?)
                .uri(Uri::from_str(request.path.unwrap())?)
                .version(Version::HTTP_11);

            for header in request.headers {
                builder = builder.header(header.name, header.value);
            }

            Ok(Some(builder))
        }
        Ok(httparse::Status::Partial) => Ok(None),
        Err(e) => Err(ParseRequestError::InvalidByte(e)),
    }
}

/// Serializes the response parts to a raw HTTP response.
pub fn serialize_parts_to_bytes(parts: Parts) -> Vec<u8> {
    let mut serialized = Vec::new();

    serialized.extend_from_slice(
        format!(
            "HTTP/1.1 {} {}\r\n",
            parts.status,
            parts.status.canonical_reason().unwrap()
        )
        .as_bytes(),
    );

    for (key, value) in parts.headers.iter() {
        serialized
            .extend_from_slice(format!("{}: {}\r\n", key, value.to_str().unwrap()).as_bytes());
    }

    let date = Zoned::now()
        .strftime("date: %a, %d %b %Y %H:%M:%S GMT\r\n")
        .to_string();
    serialized.extend_from_slice(date.as_bytes());
    serialized.extend_from_slice(b"\r\n");

    serialized
}
