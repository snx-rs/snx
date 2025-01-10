mod request;
mod response;
pub mod router;

use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

use http::StatusCode;
use rayon::ThreadPool;
use request::parse_request;
use response::serialize_to_stream;
use router::Router;

pub use http::{Request, Response};

const HEADER_BUFFER_SIZE: usize = 8192;

/// Starts serving HTTP requests on the given address.
///
/// Accepts incoming connections, handles and parses incoming HTTP requests, dispatches them to the
/// given router, executes the associated controller actions and sends back responses.
pub fn serve<A: ToSocketAddrs>(
    addr: A,
    pool: ThreadPool,
    router: Router,
) -> Result<(), ServeError> {
    let listener = TcpListener::bind(addr)?;

    for stream in listener.incoming() {
        let stream = stream?;

        pool.install(|| handle_connection(stream, &router));
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum ServeError {
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Handles an incoming connection.
///
/// Reads data from the stream, parses it into a [Request], dispatches it to the router,
/// executes the associated controller action and writes a response to the stream.
fn handle_connection(mut stream: TcpStream, router: &Router) {
    let mut buffer = [0; HEADER_BUFFER_SIZE];

    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            if bytes_read == 0 {
                return;
            }

            let response = match parse_request(&buffer) {
                Ok(request) => {
                    let response = match router.dispatch(&request) {
                        Some((route, _)) => (route.handler)(request.clone()),
                        None => Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(())
                            .unwrap(),
                    };

                    tracing::info!(
                        name: "request", "{} - \"{} {} {:?}\" {}",
                        stream.peer_addr().unwrap(),
                        request.method(),
                        request.uri(),
                        request.version(),
                        response.status().as_u16(),
                    );

                    response
                }
                Err(e) => {
                    tracing::warn!("request could not be parsed: {e}");

                    Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body(())
                        .unwrap()
                }
            };

            let _ = stream.write_all(&serialize_to_stream(&response));
        }
        Err(e) => {
            tracing::warn!(
                "incoming stream could not be read from {:?}: {e}",
                stream.peer_addr()
            );
        }
    }
}
