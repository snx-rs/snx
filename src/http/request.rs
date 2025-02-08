use std::{collections::HashMap, net::SocketAddr, str};

use super::{header::Header, Method};

/// The maxmimum amount of headers that will be parsed.
const HEADERS_COUNT: usize = 32;

/// Represents an HTTP request.
#[derive(Clone, Debug)]
pub struct Request {
    method: Method,
    path: String,
    body: Vec<u8>,
    headers: Vec<Header>,
    peer_addr: Option<SocketAddr>,
    pub params: Option<HashMap<String, String>>,
}

impl Request {
    /// Creates a new builder-style object to manufacture a Request.
    ///
    /// ```
    /// use snx::{request::Request, Method};
    ///
    /// let request = Request::builder()
    ///     .method(Method::Get)
    ///     .path("/")
    ///     .header("Content-Type", "application/json")
    ///     .body(vec![])
    ///     .build();
    /// ```
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Gets the HTTP method for this request.
    ///
    /// ```
    /// use snx::{request::Request, Method};
    ///
    /// let request = Request::builder().method(Method::Post).build();
    /// let method = request.method();
    /// ```
    pub fn method(&self) -> Method {
        self.method.clone()
    }

    /// Gets the path for this request.
    ///
    /// ```
    /// use snx::request::Request;
    ///
    /// let request = Request::builder().path("/").build();
    /// let path = request.path();
    /// ```
    pub fn path(&self) -> String {
        self.path.clone()
    }

    /// Gets the peer address for this request.
    ///
    /// ```
    /// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    ///
    /// use snx::request::Request;
    ///
    /// let request = Request::builder()
    ///     .peer_addr(Some(SocketAddr::new(
    ///        IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    ///        8080
    ///      )))
    ///     .build();
    /// let peer_address = request.peer_addr();
    /// ```
    pub fn peer_addr(&self) -> Option<SocketAddr> {
        self.peer_addr
    }

    /// Tries to parse a request object from a buffer of bytes.
    ///
    /// ```
    /// use snx::request::Request;
    ///
    /// let buffer = [0; 8192];
    /// let request = Request::try_parse_from_bytes(&buffer, None);
    /// ```
    pub fn try_parse_from_bytes(
        buffer: &[u8],
        peer_addr: Option<SocketAddr>,
    ) -> Result<Self, ParseRequestError> {
        let mut headers = [httparse::EMPTY_HEADER; HEADERS_COUNT];
        let mut req = httparse::Request::new(&mut headers);

        let mut request = Request::builder().peer_addr(peer_addr);

        match req.parse(buffer) {
            Ok(httparse::Status::Complete(_)) => {
                let method_str = req.method.ok_or(ParseRequestError::MissingMethod)?;
                let path = req.path.ok_or(ParseRequestError::MissingPath)?;

                let method = Method::from(method_str);
                request = request.method(method).path(path);

                for header in req.headers.iter() {
                    let name = header.name.to_string();
                    let value = str::from_utf8(header.value)?.to_string();

                    request = request.header(&name, &value);
                }

                // TODO: temporarily read entire body into request.body
                request = request.body(vec![]);

                Ok(request.build())
            }
            Ok(httparse::Status::Partial) => Err(ParseRequestError::Partial),
            Err(e) => Err(e.into()),
        }
    }
}

/// Represents an error that occurred during request parsing, this will result is a 400 Bad Request
/// being sent to the client.
#[derive(thiserror::Error, Debug)]
pub enum ParseRequestError {
    #[error("method is missing")]
    MissingMethod,
    #[error("path is missing")]
    MissingPath,
    #[error("header value is invalid utf-8")]
    InvalidUtf8HeaderValue(#[from] str::Utf8Error),
    #[error("partial request")]
    Partial,
    #[error(transparent)]
    General(#[from] httparse::Error),
}

/// An HTTP request builder.
pub struct Builder {
    method: Method,
    path: String,
    body: Vec<u8>,
    headers: Vec<Header>,
    peer_addr: Option<SocketAddr>,
    params: Option<HashMap<String, String>>,
}

impl Builder {
    /// Creates a new default instance of the request builder.
    ///
    /// ```
    /// use snx::request;
    ///
    /// let builder = request::Builder::new();
    /// ```
    pub fn new() -> Self {
        Builder::default()
    }

    /// Sets the HTTP method for this request.
    ///
    /// ```
    /// use snx::{request, Method};
    ///
    /// let builder = request::Builder::new().method(Method::Post);
    /// ```
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;

        self
    }

    /// Sets the path for this request.
    ///
    /// ```
    /// use snx::{request, Method};
    ///
    /// let builder = request::Builder::new().path("/");
    /// ```
    pub fn path(mut self, path: &str) -> Self {
        self.path = path.to_string();

        self
    }

    /// Sets the body for this request.
    ///
    /// ```
    /// use snx::{request, Method};
    ///
    /// let builder = request::Builder::new().body(vec![]);
    /// ```
    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;

        self
    }

    /// Adds a header to this request.
    ///
    /// ```
    /// use snx::request;
    ///
    /// let builder = request::Builder::new().header("Accept-Encoding", "gzip, deflate, br");
    /// ```
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.headers.push((key, value).into());

        self
    }

    /// Sets the peer address for this request.
    ///
    /// ```
    /// use std::net::{SocketAddr, IpAddr, Ipv4Addr};
    ///
    /// use snx::request;
    ///
    /// let builder = request::Builder::new().peer_addr(Some(SocketAddr::new(
    ///         IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
    ///         8080,
    ///     )));
    /// ```
    pub fn peer_addr(mut self, peer_addr: Option<SocketAddr>) -> Self {
        self.peer_addr = peer_addr;

        self
    }

    /// Builds the HTTP request.
    ///
    /// ```
    /// use snx::request;
    ///
    /// let request = request::Builder::new().build();
    /// ```
    pub fn build(&self) -> Request {
        Request {
            peer_addr: self.peer_addr,
            method: self.method.clone(),
            path: self.path.clone(),
            body: self.body.clone(),
            headers: self.headers.clone(),
            params: Default::default(),
        }
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            method: Method::Get,
            path: "/".to_string(),
            body: vec![],
            headers: vec![],
            params: Default::default(),
            peer_addr: None,
        }
    }
}
