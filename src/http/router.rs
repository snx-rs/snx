use std::{collections::HashMap, sync::Arc};

use http::{Method, Request, Response};

const DYNAMIC_CHARS: [char; 3] = ['{', '}', '*'];

#[derive(Clone)]
pub struct Route {
    method: Method,
    path: &'static str,
    pub handler: Arc<dyn Fn(Request<()>) -> Response<()> + Send + Sync>,
}

impl Route {
    /// Creates a new `GET` route.
    pub fn get<F>(path: &'static str, handler: F) -> Self
    where
        F: Fn(Request<()>) -> Response<()> + Send + Sync + 'static,
    {
        Self {
            method: Method::GET,
            path,
            handler: Arc::new(handler),
        }
    }
}

/// Used to build a [Router].
pub struct RouterBuilder {
    routes: Vec<Route>,
}

impl RouterBuilder {
    /// Constructs a new [RouterBuilder].
    pub fn new() -> Self {
        Self { routes: vec![] }
    }

    /// Adds a route to the router.
    pub fn add_route(mut self, route: Route) -> Self {
        self.routes.push(route);

        self
    }

    /// Adds multiple routes to the router.
    pub fn add_routes(mut self, routes: &[Route]) -> Self {
        self.routes.extend_from_slice(routes);

        self
    }

    /// Builds the [Router].
    pub fn build(mut self) -> Router {
        self.sort_routes();

        Router {
            routes: self.routes,
        }
    }

    /// Sorts the routes so that static ones come before dynamic ones.
    fn sort_routes(&mut self) {
        self.routes.sort_by(|a, b| {
            (!b.path.contains(DYNAMIC_CHARS)).cmp(&!a.path.contains(DYNAMIC_CHARS))
        });
    }
}

impl Default for RouterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Routes requests to the correct handler.
///
/// ```
/// use snx::{router::{Route, Router}, Response};
///
/// let route = Route::get("/", |_| Response::builder().body(()).unwrap());
/// Router::builder().add_route(route).build();
/// ```
pub struct Router {
    routes: Vec<Route>,
}

impl Router {
    /// Constructs a new [RouterBuilder].
    pub fn builder() -> RouterBuilder {
        RouterBuilder::new()
    }

    /// Dispatches a [Request] to the [Router], tries to find a matching [Route] and returns
    /// it with possible parameters.
    pub fn dispatch<T>(&self, request: &Request<T>) -> Option<(Route, HashMap<String, String>)> {
        let mut params = HashMap::new();

        let segments: for<'a> fn(&'a str) -> Vec<&'a str> =
            |s| s.split('/').filter(|p| !p.is_empty()).collect();

        'outer: for route in self.routes.iter() {
            let route_segments = segments(route.path);
            let request_segments = segments(request.uri().path());

            for (route_seg, req_seg) in route_segments.iter().zip(request_segments.iter()) {
                if *route_seg == "*" {
                    unimplemented!("handle wildcard in route path");
                } else if route_seg.starts_with('{') && route_seg.ends_with('}') {
                    let name = route_seg
                        .strip_prefix('{')
                        .unwrap()
                        .strip_suffix('}')
                        .unwrap();

                    params.insert(name.to_string(), (*req_seg).to_string());
                } else if route_seg != req_seg {
                    continue 'outer;
                }
            }

            if route_segments.len() > request_segments.len() {
                if route_segments[request_segments.len()] == "*" {
                    todo!("handle wildcard at end of route");
                }

                continue;
            }

            if request_segments.len() > route_segments.len() {
                continue;
            }

            return Some((route.clone(), params));
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_correctly_matches_routes() {
        let cases = HashMap::from([
            ("/", "/"),
            ("/posts", "/posts"),
            ("/posts/5", "/posts/{id}"),
            ("/posts/not-found", "/posts/not-found"),
            ("/posts/5/comments", "/posts/{id}/comments"),
            (
                "/posts/2025/01/07/my-first-post",
                "/posts/2025/01/07/my-first-post",
            ),
            (
                "/posts/2025/01/07/my-first-post",
                "/posts/{year}/{month}/{day}/{slug}",
            ),
        ]);

        let handler = |_| Response::builder().body(()).unwrap();

        let router = Router::builder()
            .add_routes(
                &cases
                    .values()
                    .map(|v| Route::get(v, handler))
                    .collect::<Vec<Route>>(),
            )
            .build();

        for (path, route) in cases.iter() {
            let request = Request::builder().uri(*path).body(()).unwrap();
            let result = router.dispatch(&request).unwrap();

            assert_eq!(route, &result.0.path);
        }
    }
}
