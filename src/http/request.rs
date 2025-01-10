use std::str::FromStr;

use http::{method::InvalidMethod, HeaderName, HeaderValue, Method, Request};

const HEADERS_COUNT: usize = 32;

/// Tries to parse a buffer into a [Request].
///
/// - Logs and ignores unparsable headers
/// - Does not parse the body or cookies yet (that's up to extractors)
pub fn parse_request(buffer: &[u8]) -> Result<Request<()>, ParseRequestError> {
    let mut headers = [httparse::EMPTY_HEADER; HEADERS_COUNT];
    let mut req = httparse::Request::new(&mut headers);

    match req.parse(buffer) {
        Ok(httparse::Status::Complete(_)) => {
            let method = req.method.ok_or(ParseRequestError::MissingMethod)?;
            let path = req.path.ok_or(ParseRequestError::MissingPath)?;

            let mut request = Request::builder()
                .method(Method::from_str(method).map_err(ParseRequestError::InvalidMethod)?)
                .uri(path);

            for header in req.headers.iter() {
                match (
                    header.name.parse::<HeaderName>(),
                    HeaderValue::from_bytes(header.value),
                ) {
                    (Ok(name), Ok(value)) => {
                        request = request.header(name, value);
                    }
                    _ => {
                        tracing::warn!("header could not be parsed: ({:?})", header);
                    }
                }
            }

            Ok(request.body(())?)
        }
        Ok(httparse::Status::Partial) => Err(ParseRequestError::Partial),
        Err(e) => Err(e.into()),
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseRequestError {
    #[error("method is missing")]
    MissingMethod,
    #[error("path is missing")]
    MissingPath,
    #[error(transparent)]
    InvalidMethod(InvalidMethod),
    #[error("partial request")]
    Partial,
    #[error(transparent)]
    General(#[from] httparse::Error),
    #[error(transparent)]
    RequestCreation(#[from] http::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_correctly_parses_a_request() {
        let request = Request::builder()
            .method(Method::DELETE)
            .uri("/posts/5")
            .header("content-type", "application/json")
            .body(())
            .unwrap();

        let actual =
            parse_request(b"DELETE /posts/5 HTTP/1.1\r\ncontent-type: application/json\r\n\r\n")
                .unwrap();

        assert_eq!(request.method(), actual.method());
        assert_eq!(request.uri(), actual.uri());
        assert_eq!(request.headers(), actual.headers());
    }
}
