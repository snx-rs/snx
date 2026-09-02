use std::io::{self};

use bytes::Bytes;
use futures::{AsyncWrite, AsyncWriteExt, Stream, StreamExt, TryStreamExt};
use http::{
    StatusCode,
    into_response::{IntoBody, IntoResponse},
};

use crate::{App, handler::trigger, request::Request, router::RouterError};

const MAX_BYTES: usize = 8192;

/// Handles an incoming stream of bytes, turns it into a Request object, runs it through the router,
/// executes the associated handler and writes the response back to the output stream.
///
/// Parses the head part of the request and builds a [http_ext::Request] from it with a stream of
/// the remaining bytes as the body. ...
pub async fn serve<I, O>(app: &App, mut input: I, mut output: O)
where
    I: Stream<Item = io::Result<Bytes>> + Unpin + Send + 'static,
    O: AsyncWrite + Unpin,
{
    let mut read: Vec<u8> = Vec::new();
    while let Ok(Some(bytes)) = input.try_next().await {
        read.extend_from_slice(&bytes);
        let response = match bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            Some(pos) => match http::parse_request(&bytes.clone().split_to(pos + 4)) {
                Ok(Some(req)) => {
                    let body = futures::stream::iter(std::iter::once(Ok(bytes.slice((pos + 4)..))))
                        // .chain(input)
                        .into_body();

                    match req.body(body) {
                        Ok(req) => {
                            match app
                                .router
                                .at(req.method(), "localhost", &req.uri().to_string())
                            {
                                Ok(matched) => {
                                    trigger(matched.route.handler, Request::new(req)).await
                                }
                                Err(RouterError::NotFound) => Box::new(StatusCode::NOT_FOUND),
                                Err(RouterError::MethodNotAllowed) => {
                                    Box::new(StatusCode::METHOD_NOT_ALLOWED)
                                }
                            }
                        }
                        Err(err) => {
                            tracing::debug!(target = "snx::server", error = %err, "failed to parse incoming request");
                            Box::new(StatusCode::BAD_REQUEST)
                        }
                    }
                }
                Ok(None) => unreachable!(),
                Err(err) => {
                    tracing::debug!(target = "snx::server", error = %err, "failed to parse incoming request");
                    Box::new(StatusCode::BAD_REQUEST)
                }
            },
            None if bytes.len() > MAX_BYTES => {
                Box::new(StatusCode::REQUEST_HEADER_FIELDS_TOO_LARGE)
            }
            None => continue,
        };

        let (parts, mut body) = response.into_response().into_parts();
        let head = http::serialize_parts_to_bytes(parts);
        let _ = output.write(&head).await.ok();
        while let Some(Ok(bytes)) = body.next().await {
            let _ = output.write(&bytes).await.ok();
        }
        return;
    }
}
