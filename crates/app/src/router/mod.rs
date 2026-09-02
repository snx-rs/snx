mod builder;

use std::{
    collections::{BTreeMap, HashMap},
    fmt,
    sync::Arc,
};

use http_ext::Method;
use regex::Regex;

use crate::handler::Handler;
pub use crate::router::builder::Builder;

/// Represents a route in the router.
#[derive(Clone)]
pub struct Route {
    pub method: Method,
    pub path: String,
    pub host: String,
    pub handler: Arc<Box<dyn Handler + Send + Sync>>,
    // pub middleware: Vec<Middleware>,
}

#[derive(Debug)]
pub struct MatchedRoute {
    pub route: Route,
    pub parameters: HashMap<String, String>,
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route")
            .field("method", &self.method)
            .field("path", &self.path)
            .field("host", &self.host)
            .finish()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RouterError {
    NotFound,
    MethodNotAllowed,
}

#[derive(Clone, Debug)]
pub struct Router {
    pub hosts: BTreeMap<String, (Regex, HashMap<Method, matchit::Router<Route>>)>,
}

impl Router {
    /// Creates a new builder-style object used to manufacture a router.
    pub fn builder(host: String) -> Builder {
        Builder::new(host)
    }

    /// Tries to find a route matching the given criteria and returns it with its path and host
    /// parameters.
    pub fn at(&self, method: &Method, host: &str, path: &str) -> Result<MatchedRoute, RouterError> {
        // find the host by executing regexes
        let (key, (pattern, methods)) = self
            .hosts
            .iter()
            .find(|(_, (pattern, _))| pattern.captures(host).is_some())
            .ok_or(RouterError::NotFound)?;

        let captures = pattern.captures(host).ok_or(RouterError::NotFound)?;

        // do the path routing using matchit and search for routes with different method to return
        // method not allowed errors
        let route = methods
            .get(method)
            .and_then(|router| router.at(path).ok())
            .ok_or(match self.find_alternatives(path, methods) {
                true => RouterError::MethodNotAllowed,
                false => RouterError::NotFound,
            })?;

        // capture and add host and route parameters
        let compiled_host_regex = compile_host_pattern(key);
        let mut parameters = HashMap::new();
        parameters.extend(
            compiled_host_regex
                .capture_names()
                .flatten()
                .filter_map(|name| captures.name(name).map(|m| (name, m)))
                .map(|(name, m)| (name.to_string(), m.as_str().to_string())),
        );
        parameters.extend(
            route
                .params
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string())),
        );

        Ok(MatchedRoute {
            route: route.value.clone(),
            parameters,
        })
    }

    /// Returns whether or not a route exists in the method router for the given path.
    fn find_alternatives(
        &self,
        path: &str,
        methods: &HashMap<Method, matchit::Router<Route>>,
    ) -> bool {
        methods.iter().any(|router| router.1.at(path).is_ok())
    }
}

/// Compiles a regular expression that captures dynamic components of a hostname.
pub fn compile_host_pattern(pattern: &str) -> Regex {
    let mut regex_pattern = regex::escape(pattern);

    regex_pattern = regex_pattern
        .replace(r"\{", "(?P<")
        .replace(r"\}", ">[^.]+)");
    regex_pattern = regex_pattern.replace(r"\*", "[^.]+");

    Regex::new(&format!("^{}$", regex_pattern)).expect("Invalid regex")
}
