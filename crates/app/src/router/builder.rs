use crate::format::Format;
use crate::handler::{Handler, HandlerFn};
use crate::request::Request;
use crate::router::Route;
use crate::router::Router;
use crate::router::compile_host_pattern;
use crate::validation::Validatable;
use http::into_response::IntoResponse;
use http_ext::Method;
use regex::Regex;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::btree_map::Entry;
use std::sync::Arc;

/// Defines a method to add routes to the builder for the given HTTP method.
macro_rules! define_route_method {
    ($k:ident, $v:ident) => {
        /// Adds a route to the builder.
        // pub fn $k(mut self, path: &'static str, handler: impl Handler + 'static) -> Self {
        pub fn $k<Func, F, Fut, R>(mut self, path: &'static str, handler: Func) -> Self
        where
            Func: Fn(Request<F>) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = R> + Send + 'static,
            F: Format + Send + Sync + 'static,
            F::Value: Validatable,
            R: IntoResponse + 'static,
        {
            self.routes.push(Route {
                method: Method::$v,
                path: path.to_string(),
                host: self.host.clone(),
                handler: Arc::new(Box::new(HandlerFn::new(handler)) as Box<dyn Handler>),
                // middleware: Default::default(),
            });

            self
        }
    };
}

#[derive(Clone, Default)]
pub struct Builder {
    host: String,
    prefix: Option<String>,
    // middleware: Vec<Middleware>,
    routes: Vec<Route>,
    children: Vec<Builder>,
}

impl Builder {
    /// Creates a new builder with the given host.
    pub fn new(host: String) -> Self {
        Self {
            host,
            ..Default::default()
        }
    }

    /// Adds a new route group with a host to the builder.
    pub fn host(mut self, host: &'static str, body: impl Fn(Builder) -> Builder) -> Self {
        let mut builder = Self {
            host: host.to_string(),
            ..Default::default()
        };

        builder = body(builder);
        self.children.push(builder);

        self
    }

    /// Adds a new route group with a prefix to the builder.
    pub fn prefix(mut self, prefix: &'static str, body: impl Fn(Builder) -> Builder) -> Self {
        let mut builder = Self {
            host: self.host.clone(),
            prefix: Some(prefix.trim_matches('/').to_string()),
            ..Default::default()
        };

        builder = body(builder);
        self.children.push(builder);

        self
    }

    // /// Adds a new route group with middleware to the builder.
    // pub fn middleware(
    //     mut self,
    //     middleware: &'static [impl Fn(
    //         Request,
    //         Box<dyn Fn(Request) -> Box<dyn IntoResponse>>,
    //     ) -> Box<dyn IntoResponse>
    //               + Send
    //               + Sync],
    //     body: impl Fn(Builder) -> Builder,
    // ) -> Self {
    //     let mut builder = Self {
    //         host: self.host.clone(),
    //         ..Default::default()
    //     };
    //
    //     builder.middleware.extend(middleware.iter().map(|m| {
    //         Arc::new(Box::new(m)
    //             as Box<
    //                 dyn Fn(
    //                         Context,
    //                         Request,
    //                         Box<dyn Fn(Request) -> Box<dyn IntoResponse>>,
    //                     ) -> Box<dyn IntoResponse>
    //                     + Send
    //                     + Sync,
    //             >)
    //     }));
    //     self.children.push(body(builder));
    //
    //     self
    // }

    /// Builds the router.
    pub fn build(self) -> Result<Router, matchit::InsertError> {
        let mut hosts = BTreeMap::new();

        // for route in self.resolve(&mut vec![], &mut vec![]) {
        for route in self.resolve(&mut vec![]) {
            match hosts.entry(route.host.clone()) {
                Entry::Occupied(mut e) => {
                    let (_, methods): &mut (Regex, HashMap<Method, matchit::Router<Route>>) =
                        e.get_mut();

                    if let Some(router) = methods.get_mut(&route.method) {
                        router.insert(route.path.clone(), route.clone())?;
                    } else {
                        let mut router = matchit::Router::new();
                        router.insert(route.path.clone(), route.clone())?;
                        methods.insert(route.method, router);
                    }
                }
                Entry::Vacant(e) => {
                    let mut host_router = matchit::Router::new();
                    host_router.insert(route.path.clone(), route.clone())?;
                    let pattern = compile_host_pattern(&route.host);
                    let methods = HashMap::from([(route.method, host_router)]);

                    e.insert((pattern, methods));
                }
            }
        }

        // TODO: make sure hosts are sorted in btreemap

        Ok(Router { hosts })
    }

    /// Recursively adds (compounding) prefixes and middleware to all of this builder's children,
    /// combines the children's routes into its own and returns them.
    fn resolve(
        mut self,
        prefixes: &mut Vec<String>,
        // middleware: &mut Vec<Middleware>,
    ) -> Vec<Route> {
        if let Some(prefix) = self.prefix {
            prefixes.push(prefix);
        }

        // middleware.extend(self.middleware);

        for route in &mut self.routes {
            if route.path.len() > 1
                && let Some(path) = route.path.strip_suffix('/')
            {
                route.path = path.to_string();
            }

            route.path = format!("{}{}", prefixes.join("/"), route.path);
            // route.middleware.extend(middleware.clone());
            route.host = self.host.clone();
        }

        for child in self.children {
            self.routes
                // .extend_from_slice(&child.resolve(&mut prefixes.clone(), &mut middleware.clone()));
                .extend_from_slice(&child.resolve(&mut prefixes.clone()));
        }

        self.routes
    }

    define_route_method!(get, GET);
    define_route_method!(head, HEAD);
    define_route_method!(post, POST);
    define_route_method!(put, PUT);
    define_route_method!(delete, DELETE);
    define_route_method!(connect, CONNECT);
    define_route_method!(options, OPTIONS);
    define_route_method!(trace, TRACE);
    define_route_method!(patch, PATCH);
}
