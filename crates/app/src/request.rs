use std::{io, marker::PhantomData, num};

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

    /// Reads the incoming HTTP body, deserializes it into the given type and validates it
    pub async fn payload(&mut self) -> Result<F::Value, PayloadError<F::Error>> {
        let body = self.read_body_to_end().await.unwrap();
        println!("bytes: {:?}", body);
        println!("utf8: {:?}", str::from_utf8(&body).unwrap());
        let value = F::from_bytes(&body).map_err(PayloadError::Deserialize)?;
        value.validate().map_err(PayloadError::Validation)

        // let mut bytes = Vec::new();
        // let length = self.inner.headers().get("Content-Length").unwrap().clone();
        // loop {
        //     println!("PART utf8: {:?}", str::from_utf8(&bytes).unwrap());
        //     match self.inner.body_mut().try_next().await.unwrap() {
        //         Some(b) => {
        //             bytes.extend_from_slice(&b);
        //             if b.len() >= length.to_str().unwrap().parse::<usize>().unwrap() {
        //                 break;
        //             }
        //         }
        //         None => break,
        //     }
        // }
        // println!("bytes: {:?}", bytes);
        // println!("utf8: {:?}", str::from_utf8(&bytes).unwrap());
        // read until end
        // while let Some(b) = self
        //     .inner
        //     .body_mut()
        //     .try_next()
        //     .await
        //     .map_err(PayloadError::Read)?
        // {
        //     println!("a");
        //     bytes.extend_from_slice(&b);
        // }
        // println!("b");
    }

    async fn read_body_to_end(&mut self) -> Result<Vec<u8>, ReadBodyError> {
        let length = self
            .inner
            .headers()
            .get("content-length")
            .ok_or(ReadBodyError::ContentLengthMissing)?
            .to_str()?
            .parse::<usize>()?;

        todo!();

        // let mut buf = Vec::with_capacity(length);
        // self.inner
        //     .body_mut()
        //     .into_async_read()
        //     .read_exact(&mut buf)
        //     .await?;

        // Ok(buf)
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
    Request,
    Read(io::Error),
    Deserialize(E),
    Validation(validation::Error),
}
