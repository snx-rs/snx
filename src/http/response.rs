use http::Response;

/// Serializes a [Response] to a vec of bytes.
pub fn serialize_to_stream<T>(response: &Response<T>) -> Vec<u8> {
    let mut serialized = Vec::new();

    // status line
    serialized.extend_from_slice(
        format!(
            "HTTP/1.1 {} {}\r\n",
            response.status().as_u16(),
            response.status().canonical_reason().unwrap_or("")
        )
        .as_bytes(),
    );

    // headers
    for (key, value) in response.headers() {
        serialized
            .extend_from_slice(format!("{}: {}\r\n", key, value.to_str().unwrap_or("")).as_bytes());
    }
    serialized.extend_from_slice(b"\r\n");

    serialized
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_correctly_serializes_an_empty_response() {
        let response = Response::builder().body(()).unwrap();

        let expected = b"HTTP/1.1 200 OK\r\n\r\n".to_vec();
        let actual = serialize_to_stream(&response);

        assert_eq!(expected, actual);
    }

    #[test]
    fn it_correctly_serializes_a_response_with_some_headers_and_without_a_body() {
        let response = Response::builder()
            .header("content-type", "application/json")
            .header("last-modified", "Tue, 07 Jan 2025 15:09:42 GMT")
            .body(())
            .unwrap();

        let expected = b"HTTP/1.1 200 OK\r\ncontent-type: application/json\r\nlast-modified: Tue, 07 Jan 2025 15:09:42 GMT\r\n\r\n".to_vec();
        let actual = serialize_to_stream(&response);

        assert_eq!(expected, actual);
    }
}
