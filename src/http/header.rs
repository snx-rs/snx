/// Represents an HTTP header.
#[derive(Clone, Debug)]
pub struct Header(pub String, pub String);

impl<K, V> From<(K, V)> for Header
where
    K: ToString,
    V: ToString,
{
    fn from(value: (K, V)) -> Self {
        Self(value.0.to_string(), value.1.to_string())
    }
}
