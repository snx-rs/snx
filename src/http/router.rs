use std::{
    collections::{hash_map::Entry, HashMap},
    sync::Arc,
};

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
    handler: Arc<Box<dyn Handler + Send + Sync>>,
    middleware: Vec<MiddlewareHandler>,
}

impl Route {
    /// Gets the handler for this route.
    ///
    /// ```
    /// use snx::{request::Request, router::Router};
    ///
    /// let mut request = Request::builder().path("/").build();
    /// let router = Router::builder()
    ///     .get("/", |_| "hello world!")
    ///     .build()
    ///     .unwrap();
    ///
    /// let route = router.dispatch(&mut request).unwrap();
    /// let handler = route.handler();
    /// ```
    pub fn handler(&self) -> Arc<Box<dyn Handler + Send + Sync>> {
        self.handler.clone()
    }

    /// Gets the middleware for this route.
    ///
    /// ```
    /// use snx::{request::Request, router::Router};
    ///
    /// let mut request = Request::builder().path("/").build();
    /// let router = Router::builder()
    ///     .get("/", |_| "hello world!")
    ///     .build()
    ///     .unwrap();
    ///
    /// let route = router.dispatch(&mut request).unwrap();
    /// let middleware = route.middleware();
    /// ```
    pub fn middleware(&self) -> Vec<MiddlewareHandler> {
        self.middleware.clone()
    }
}

/// Routes a [Request] to the correct handler.
pub struct Router {
    method_routers: HashMap<Method, matchit::Router<Route>>,
}

impl Router {
    /// Creates a new builder-style object to manufacture a router.
    ///
    /// ```
    /// use snx::router::Router;
    ///
    /// let builder = Router::builder();
    /// ```
    pub fn builder() -> Builder {
        Builder::new()
    }

    /// Dispatches a [Request] to the [Router], tries to find a matching [Route] and returns it
    /// with its parameters.
    ///
    /// ```
    /// use snx::{request::Request, router::Router};
    ///
    /// let mut request = Request::builder().path("/").build();
    /// let router = Router::builder()
    ///     .get("/", |_| "hello world!")
    ///     .build()
    ///     .unwrap();
    ///
    /// let route = router.dispatch(&mut request);
    /// ```
    pub fn dispatch(&self, request: &mut Request) -> Result<Route, matchit::MatchError> {
        let router = self
            .method_routers
            .get(&request.method())
            .ok_or(matchit::MatchError::NotFound)?;

        let path = request.path();
        let route = router.at(&path)?;

        let mut params = HashMap::new();
        for (key, value) in route.params.iter() {
            params.insert(key.to_string(), value.to_string());
        }
        request.params = Some(params);

        Ok(route.value.clone())
    }
}

/// Defines a method for adding routes to the Builder with the given method.
macro_rules! define_route_method {
    ($k:ident, $v:ident) => {
        /// Adds a route to the builder.
        pub fn $k(mut self, path: &'static str, handler: impl Handler + 'static) -> Self {
            self.routes.push(Route {
                method: Method::$v,
                path: path.to_string(),
                handler: Arc::new(Box::new(handler) as Box<dyn Handler>),
                middleware: Default::default(),
            });

            self
        }
    };
}

/// A router builder.
#[derive(Default)]
pub struct Builder {
    prefix: Option<&'static str>,
    children: Vec<Builder>,
    middleware: Vec<MiddlewareHandler>,
    routes: Vec<Route>,
}

impl Builder {
    /// Creates a new default instance of the router builder.
    ///
    /// ```
    /// use snx::router;
    ///
    /// let builder = router::Builder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a new group with a prefix to the builder.
    ///
    /// ```
    /// use snx::router::Router;
    ///
    /// let router = Router::builder()
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
        let mut builder = Self::new();
        builder.prefix = Some(prefix);

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
    /// let router = Router::builder()
    ///     .middleware(&[my_middleware], |router| {
    ///         router.get("/", |_| "hello world!")
    ///     })
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn middleware(
        mut self,
        middleware: &'static [impl Fn(Request, Box<dyn Fn() -> Response>) -> Box<dyn IntoResponse>
                      + Send
                      + Sync],
        body: impl Fn(Builder) -> Builder,
    ) -> Self {
        let mut builder = Self::new();

        for handler in middleware {
            builder.middleware.push(Arc::new(Box::new(handler)
                as Box<
                    dyn Fn(Request, Box<dyn Fn() -> Response>) -> Box<dyn IntoResponse>
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

    /// Recursively adds (compounding) prefixes and middleware to all of this builders children.
    pub fn resolve(
        &mut self,
        mut prefixes: Vec<&'static str>,
        mut middleware: Vec<MiddlewareHandler>,
    ) -> &Builder {
        if let Some(prefix) = self.prefix {
            prefixes.push(prefix)
        }
        middleware.extend(self.middleware.clone());

        for route in &mut self.routes {
            route.middleware.extend(middleware.clone());
            route.path = format!("{}{}", prefixes.join("/"), route.path);
        }

        for child in &mut self.children {
            let resolved = child.resolve(prefixes.clone(), middleware.clone());
            for route in resolved.routes.clone() {
                self.routes.push(route);
            }
        }

        self
    }

    /// Builds the router.
    ///
    /// ```
    /// use snx::router;
    ///
    /// let router = router::Builder::new().build();
    /// ```
    pub fn build(mut self) -> Result<Router, matchit::InsertError> {
        let mut method_routers: HashMap<Method, matchit::Router<Route>> = HashMap::new();

        let prefixes = vec![];
        let middleware = vec![];

        self.resolve(prefixes, middleware);

        for route in self.routes {
            if let Entry::Vacant(e) = method_routers.entry(route.method.clone()) {
                let mut router = matchit::Router::new();
                router.insert(route.path.clone(), route.clone())?;

                e.insert(router);
            } else {
                let router = method_routers.get_mut(&route.method).unwrap();
                router.insert(route.path.clone(), route)?;
            }
        }

        Ok(Router { method_routers })
    }
}
