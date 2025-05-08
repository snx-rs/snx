use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

use regex::Regex;

use crate::Context;

use super::{
    handler::Handler,
    middleware::MiddlewareHandler,
    request::Request,
    response::{IntoResponse, Response},
    Method,
};

/// Used to store routes in the router.
#[derive(Clone)]
pub struct Route {
    method: Method,
    path: String,
    host: String,
    handler: Arc<Box<dyn Handler + Send + Sync>>,
    middleware: Vec<MiddlewareHandler>,
}

impl Route {
    /// Gets a reference to the path.
    ///
    /// ```
    /// use snx::{request::Request, router::Router, Method};
    ///
    /// let mut request = Request::builder().path("/").build();
    /// let router = Router::builder("localhost")
    ///     .get("/", |_| "hello world!")
    ///     .build()
    ///     .unwrap();
    ///
    /// let matched_route = router.at(&Method::Get, "localhost", "/").unwrap();
    /// let path = matched_route.route.path();
    /// ```
    pub fn path(&self) -> String {
        self.path.clone()
    }

    /// Gets a reference to the handler.
    ///
    /// ```
    /// use snx::{request::Request, router::Router, Method};
    ///
    /// let mut request = Request::builder().path("/").build();
    /// let router = Router::builder("localhost")
    ///     .get("/", |_| "hello world!")
    ///     .build()
    ///     .unwrap();
    ///
    /// let matched_route = router.at(&Method::Get, "localhost", "/").unwrap();
    /// let handler = matched_route.route.handler();
    /// ```
    pub fn handler(&self) -> &Arc<Box<dyn Handler + Send + Sync>> {
        &self.handler
    }

    /// Gets a reference to the middleware.
    ///
    /// ```
    /// use snx::{request::Request, router::Router, Method};
    ///
    /// let router = Router::builder("localhost")
    ///     .get("/", |_| "hello world!")
    ///     .build()
    ///     .unwrap();
    ///
    /// let matched_route = router.at(&Method::Get, "localhost", "/").unwrap();
    /// let middleware = matched_route.route.middleware();
    /// ```
    pub fn middleware(&self) -> &Vec<MiddlewareHandler> {
        &self.middleware
    }
}

pub struct MatchedRoute<'a> {
    pub route: &'a Route,
    pub parameters: HashMap<String, String>,
}

/// Used to route a [Request] to the correct route.
pub struct Router {
    hosts: HashMap<String, (Regex, HashMap<Method, matchit::Router<Route>>)>,
}

impl Router {
    /// Creates a new builder-style object to manufacture a router.
    pub fn builder(host: &str) -> Builder {
        Builder {
            host: host.to_string(),
            ..Default::default()
        }
    }

    /// Tries to find routes matching the given criteria and returns the first one with its path
    /// and host parameters.
    ///
    /// ```
    /// use snx::{router::Router, Method};
    ///
    /// let router = Router::builder("localhost")
    ///     .get("/", |_| "hello, world!")
    ///     .build()
    ///     .unwrap();
    /// let matched_route = router.at(&Method::Get, "localhost", "/").unwrap();
    ///
    /// assert_eq!(&matched_route.route.path(), "/")
    /// ```
    pub fn at(&self, method: &Method, host: &str, path: &str) -> Result<MatchedRoute, RouterError> {
        for (host_key, (pattern, methods)) in &self.hosts {
            let compiled_host_regex = compile_host_pattern(host_key);

            if let Some(captures) = pattern.captures(host) {
                if let Some(router) = methods.get(method) {
                    if let Ok(route) = router.at(path) {
                        let mut parameters = HashMap::new();

                        for name in compiled_host_regex.capture_names().flatten() {
                            if let Some(m) = captures.name(name) {
                                parameters.insert(name.to_string(), m.as_str().to_string());
                            }
                        }

                        for (key, value) in route.params.iter() {
                            parameters.insert(key.to_string(), value.to_string());
                        }

                        return Ok(MatchedRoute {
                            route: route.value,
                            parameters,
                        });
                    } else {
                        return Err(match self.find_alternatives(path, methods) {
                            true => RouterError::MethodNotAllowed,
                            false => RouterError::NotFound,
                        });
                    }
                } else {
                    return Err(match self.find_alternatives(path, methods) {
                        true => RouterError::MethodNotAllowed,
                        false => RouterError::NotFound,
                    });
                }
            }
        }

        Err(RouterError::NotFound)
    }

    /// Returns whether or not a route exists in the method router for the given path.
    fn find_alternatives(
        &self,
        path: &str,
        methods: &HashMap<Method, matchit::Router<Route>>,
    ) -> bool {
        for router in methods {
            if router.1.at(path).is_ok() {
                return true;
            }
        }

        false
    }
}

#[derive(Debug)]
pub enum RouterError {
    NotFound,
    MethodNotAllowed,
}

/// Defines a method for adding routes to the Builder with the given method.
macro_rules! define_route_method {
    ($k:ident, $v:ident) => {
        /// Adds a route to the builder.
        pub fn $k(mut self, path: &'static str, handler: impl Handler + 'static) -> Self {
            self.routes.push(Route {
                method: Method::$v,
                path: path.to_string(),
                host: self.host.clone(),
                handler: Arc::new(Box::new(handler) as Box<dyn Handler>),
                middleware: Default::default(),
            });

            self
        }
    };
}

/// A builder-style object used to build a router.
#[derive(Default)]
pub struct Builder {
    host: String,
    prefix: Option<String>,
    middleware: Vec<MiddlewareHandler>,
    routes: Vec<Route>,
    children: Vec<Builder>,
}

impl Builder {
    /// Adds a new group with a prefix to the builder.
    ///
    /// ```
    /// use snx::router::Router;
    ///
    /// let router = Router::builder("localhost")
    ///     .prefix("/posts", |router| {
    ///         router
    ///             .post("/", |_| "creates a post")
    ///             .get("/", |_| "returns a list of posts")
    ///             .get("/{id}", |_| "returns a single post")
    ///             .put("/{id}", |_| "updates a post")
    ///             .delete("/{id}", |_| "deletes a post")
    ///     })
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn prefix(mut self, prefix: &'static str, body: impl Fn(Builder) -> Builder) -> Self {
        let mut builder = Self {
            host: self.host.clone(),
            prefix: Some(prefix.to_string()),
            ..Default::default()
        };

        builder = body(builder);
        self.children.push(builder);

        self
    }

    /// Adds a new group with a host to the builder.
    ///
    /// ```
    /// use snx::router::Router;
    ///
    /// let router = Router::builder("localhost")
    ///     .host("{tenant}.acme.com", |router| {
    ///         router.get("/", |_| "tenant home page here")
    ///     })
    ///     .build()
    ///     .unwrap();
    pub fn host(mut self, host: &'static str, body: impl Fn(Builder) -> Builder) -> Self {
        let mut builder = Self {
            host: host.to_string(),
            ..Default::default()
        };

        builder = body(builder);
        self.children.push(builder);

        self
    }

    /// Adds a new group with middleware to the builder.
    ///
    /// ```
    /// use snx::{router::Router, request::Request, response::{IntoResponse, Response}};
    ///
    /// fn my_middleware(_req: Request, next: Box<dyn Fn() -> Response>) -> Box<dyn IntoResponse> {
    ///     println!("you accessed my route!");
    ///
    ///     Box::new(next())
    /// };
    ///
    /// let router = Router::builder("localhost")
    ///     .middleware(&[my_middleware], |router| {
    ///         router.get("/", |_| "hello world!")
    ///     })
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn middleware(
        mut self,
        middleware: &'static [impl Fn(
            Context,
            Request,
            Box<dyn Fn(Request) -> Response>,
        ) -> Box<dyn IntoResponse>
                      + Send
                      + Sync],
        body: impl Fn(Builder) -> Builder,
    ) -> Self {
        let mut builder = Self {
            host: self.host.clone(),
            ..Self::default()
        };

        for handler in middleware {
            builder.middleware.push(Arc::new(Box::new(handler)
                as Box<
                    dyn Fn(
                            Context,
                            Request,
                            Box<dyn Fn(Request) -> Response>,
                        ) -> Box<dyn IntoResponse>
                        + Send
                        + Sync,
                >));
        }

        builder = body(builder);
        self.children.push(builder);

        self
    }

    define_route_method!(get, Get);
    define_route_method!(head, Head);
    define_route_method!(post, Post);
    define_route_method!(put, Put);
    define_route_method!(delete, Delete);
    define_route_method!(connect, Connect);
    define_route_method!(options, Options);
    define_route_method!(trace, Trace);
    define_route_method!(patch, Patch);

    /// Builds a router.
    pub fn build(self) -> Result<Router, matchit::InsertError> {
        let mut hosts = HashMap::new();

        for route in self.resolve(&mut vec![], &mut vec![]) {
            if let Entry::Vacant(e) = hosts.entry(route.host.clone()) {
                let mut router = matchit::Router::new();
                router.insert(route.path.clone(), route.clone())?;

                let mut methods = HashMap::new();
                methods.insert(route.method, router);

                let pattern = compile_host_pattern(&route.host);

                e.insert((pattern, methods));
            } else {
                let (_, methods) = hosts.get_mut(&route.host).unwrap();

                if let Some(router) = methods.get_mut(&route.method) {
                    router.insert(route.path.clone(), route.clone())?;
                } else {
                    let mut router = matchit::Router::new();
                    router.insert(route.path.clone(), route.clone())?;

                    methods.insert(route.method, router);
                }
            }
        }

        Ok(Router { hosts })
    }

    /// Recursively adds (compounding) prefixes and middleware to all of this builders' children,
    /// combines the children's routes into its own and returns them.
    fn resolve(
        mut self,
        prefixes: &mut Vec<String>,
        middleware: &mut Vec<MiddlewareHandler>,
    ) -> Vec<Route> {
        if let Some(prefix) = self.prefix {
            prefixes.push(prefix);
        }
        middleware.extend(self.middleware);

        for route in &mut self.routes {
            if prefixes.is_empty() && route.path.len() > 1 {
                route.path = route.path.trim_end_matches('/').to_string();
            }

            route.middleware.extend(middleware.clone());
            route.path = format!("{}{}", prefixes.join("/"), route.path);

            if route.path.len() > 1 {
                if let Some(path) = route.path.strip_suffix('/') {
                    route.path = path.to_string();
                }
            }

            route.host = self.host.clone();
        }

        for child in self.children {
            self.routes
                .extend_from_slice(&child.resolve(prefixes, middleware));
        }

        self.routes
    }
}

/// Compiles a regular expression that captures dynamic components of a hostname.
fn compile_host_pattern(pattern: &str) -> Regex {
    let mut regex_pattern = regex::escape(pattern);

    regex_pattern = regex_pattern
        .replace(r"\{", "(?P<")
        .replace(r"\}", ">[^.]+)");
    regex_pattern = regex_pattern.replace(r"\*", "[^.]+");

    Regex::new(&format!("^{}$", regex_pattern)).expect("Invalid regex")
}
