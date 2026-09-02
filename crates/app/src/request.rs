use std::{io, marker::PhantomData, num};

use futures::{AsyncReadExt, TryStreamExt};
use http::into_response::Body;
use http_ext::header::ToStrError;

use crate::{
    format::Format,
    validation::{self, Validatable},
};

pub struct Request<F = ()> {
    inner: http_ext::Request<Body>,
    f: PhantomData<F>,
}

impl<F> Request<F>
where
    F: Format,
    F::Value: validation::Validatable,
{
    pub fn new(inner: http_ext::Request<Body>) -> Self {
        Self {
            inner,
            f: PhantomData,
        }
    }

    /// A reference to the associated URI
    pub fn uri(&self) -> &http_ext::Uri {
        self.inner.uri()
    }

    /// Reads the entire incoming HTTP body, deserializes it into the given payload type and
    /// validates it
    ///
    /// ```rust
    /// #[derive(Deserialize, Validate)]
    /// pub struct Payload {
    ///     #[rule(ascii)]
    ///     email: String,
    /// }
    ///
    /// pub async fn send_password_reset_email(req: Request<Payload>) {
    ///     let Payload { email } = req.payload().await?;
    ///
    ///     // ...
    /// }
    /// ```
    pub async fn payload(&mut self) -> Result<F::Value, PayloadError<F::Error>> {
        let body = self.read_body_to_end().await.map_err(PayloadError::Body)?;
        let value = F::from_bytes(&body).map_err(PayloadError::Deserialize)?;
        value.validate().map_err(PayloadError::Validation)
    }

    /// Reads this request's entire body
    async fn read_body_to_end(&mut self) -> Result<Vec<u8>, ReadBodyError> {
        let length = self
            .inner
            .headers()
            .get("content-length")
            .ok_or(ReadBodyError::ContentLengthMissing)?
            .to_str()?
            .parse::<usize>()?;

        let mut buf = vec![0u8; length];
        self.inner
            .body_mut()
            .into_async_read()
            .read_exact(&mut buf)
            .await?;

        Ok(buf)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ReadBodyError {
    #[error("content length header is missing")]
    ContentLengthMissing,
    #[error(transparent)]
    ContentLengthValueToString(#[from] ToStrError),
    #[error(transparent)]
    ContentLengthToInteger(#[from] num::ParseIntError),
    #[error(transparent)]
    Io(#[from] io::Error),
}

impl Request<()> {
    pub(crate) fn retype<F>(self) -> Request<F> {
        Request {
            inner: self.inner,
            f: PhantomData,
        }
    }
}

#[derive(Debug)]
pub enum PayloadError<E> {
    Body(ReadBodyError),
    Read(io::Error),
    Deserialize(E),
    Validation(validation::Error),
}
