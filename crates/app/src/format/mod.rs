mod json;

pub use json::Json;

pub trait Format {
    type Value;

    type Error;

    fn from_bytes(bytes: &[u8]) -> Result<Self::Value, Self::Error>;
}

impl Format for () {
    type Value = ();

    type Error = ();

    fn from_bytes(_: &[u8]) -> Result<Self::Value, Self::Error> {
        Ok(())
    }
}
