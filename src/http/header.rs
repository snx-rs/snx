use std::{collections::HashMap, ops::Deref};

/// A multimap of HTTP headers to values.
///
/// Header names are case-insensitive.
#[derive(Clone, Default, Debug)]
pub struct HeaderMap(HashMap<String, Vec<String>>);

impl HeaderMap {
    /// Creates a new header map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a header.
    ///
    /// ```
    /// use snx::HeaderMap;
    ///
    /// let mut headers = HeaderMap::new();
    /// headers.insert("Content-Type", "application/json");
    /// ```
    pub fn insert(&mut self, name: &str, value: &str) {
        match self.0.get_mut(name) {
            Some(values) => values.push(value.to_string()),
            None => {
                self.0.insert(name.to_lowercase(), vec![value.to_string()]);
            }
        }
    }

    /// Gets the first value of a header if it exists.
    ///
    /// ```
    /// use snx::HeaderMap;
    ///
    /// let mut headers = HeaderMap::new();
    /// headers.insert("Content-Type", "application/json");
    ///
    /// let content_type = headers.get("Content-Type").unwrap();
    /// ```
    pub fn get(&self, name: &str) -> Option<String> {
        self.0.get(&name.to_lowercase())?.first().cloned()
    }

    /// Gets all values of a header.
    ///
    /// ```
    /// use snx::HeaderMap;
    ///
    /// let mut headers = HeaderMap::new();
    /// headers.insert("Transfer-Encoding", "chunked");
    /// headers.insert("Transfer-Encoding", "gzip");
    ///
    /// let encodings = headers.get_all("Transfer-Encoding").unwrap();
    /// ```
    pub fn get_all(&self, name: &str) -> Option<Vec<String>> {
        self.0.get(&name.to_lowercase()).cloned()
    }
}

impl From<(&str, &str)> for HeaderMap {
    fn from(val: (&str, &str)) -> Self {
        let mut map = HeaderMap::new();

        map.insert(&val.0.to_lowercase(), val.1);

        map
    }
}

impl Deref for HeaderMap {
    type Target = HashMap<String, Vec<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
