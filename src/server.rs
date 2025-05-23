use std::{
    io::{self, Read, Write},
    net, num,
    panic::{catch_unwind, AssertUnwindSafe},
    rc::Rc,
    sync::Arc,
};

use rayon::ThreadPoolBuilder;

use crate::{
    http::{
        handler::{trigger, Handler},
        request::Request,
        response::IntoResponse,
        StatusCode,
    },
    middleware::MiddlewareHandler,
    router::{Router, RouterError},
    Context,
};

/// Encapsulates functionality to serve HTTP requests.
pub struct Server {
    listener: net::TcpListener,
    router: Router,
    ctx: Context,
    global_middleware: Vec<MiddlewareHandler>,
    num_threads: Option<usize>,
}

type ChainOperator = Rc<Box<dyn Fn(Request) -> Box<dyn IntoResponse>>>;

impl Server {
    /// Binds the server to the provided address.
    pub fn try_bind(
        addr: impl net::ToSocketAddrs,
        router: Router,
        ctx: Context,
        global_middleware: Vec<MiddlewareHandler>,
    ) -> io::Result<Self> {
        let listener = net::TcpListener::bind(addr)?;

        Ok(Self {
            listener,
            num_threads: None,
            router,
            ctx,
            global_middleware,
        })
    }

    /// Starts serving incoming HTTP requests.
    pub fn serve(self) {
        let num_threads = self.num_threads.unwrap_or(
            std::thread::available_parallelism()
                .map(num::NonZero::get)
                .unwrap_or(4),
        );

        let pool = ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();

        for connection in self.listener.incoming() {
            match connection {
                Ok(stream) => pool.install(|| self.handle_connection(stream)),
                Err(e) => tracing::info!("client failed to connect: {e}"),
            }
        }
    }

    /// Sets the number of threads to be used in the threadpool.
    pub fn num_threads(mut self, amount: usize) -> Self {
        self.num_threads = Some(amount);

        self
    }

    /// Handles an incoming connection.
    ///
    /// Reads data from the stream, parses it into a [Request], dispatches it to the router,
    /// executes the associated handler and writes a response back to the stream.
    fn handle_connection(&self, mut stream: net::TcpStream) {
        let mut buffer = [0; 8192];

        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                if bytes_read == 0 {
                    return;
                }

                let into_response: Box<dyn IntoResponse> =
                    match Request::try_parse_from_bytes(&buffer, stream.peer_addr().ok()) {
                        Ok(mut request) => {
                            let host = request.headers().get("host").unwrap();
                            match self.router.at(&request.method(), &host, &request.path()) {
                                Ok(route) => {
                                    request.params = route.parameters;

                                    self.execute(
                                        route.route.handler().clone(),
                                        route.route.middleware().clone(),
                                        request,
                                    )
                                }
                                Err(RouterError::NotFound) => self.execute(
                                    Arc::new(Box::new(|_, _| StatusCode::NotFound)),
                                    vec![],
                                    request,
                                ),
                                Err(RouterError::MethodNotAllowed) => self.execute(
                                    Arc::new(Box::new(|_, _| StatusCode::MethodNotAllowed)),
                                    vec![],
                                    request,
                                ),
                            }
                        }
                        Err(e) => {
                            tracing::warn!("could not parse request: {e}");

                            Box::new(StatusCode::BadRequest)
                        }
                    };

                let _ = stream.write_all(
                    &into_response
                        .into_response()
                        .serialize_to_raw_http_response(),
                );
            }
            Err(e) => tracing::warn!("could not read from client: {e}"),
        }
    }

    /// Executes the given handler with the given middleware.
    fn execute(
        &self,
        handler: Arc<Box<dyn Handler + Send + Sync>>,
        middleware: Vec<MiddlewareHandler>,
        request: Request,
    ) -> Box<dyn IntoResponse> {
        let mut middleware = middleware.clone();
        let ctx = self.ctx.clone();
        let mut chain: Vec<ChainOperator> = Vec::with_capacity(middleware.len() + 1);

        // first, add the actual handler call to the chain (this will be called last)
        chain.push(Rc::new(Box::new(move |request: Request| {
            catch_unwind(AssertUnwindSafe(|| {
                trigger(ctx.clone(), request.clone(), handler.clone())
            }))
            .unwrap_or(Box::new(StatusCode::InternalServerError))
        })));

        // second, loop over all middleware for route (and global middleware) and add in reverse
        // order.
        middleware.extend_from_slice(&self.global_middleware);
        for handler in middleware {
            let ctx = self.ctx.clone();
            let op = chain.last().unwrap().clone();

            chain.push(Rc::new(Box::new(move |request: Request| {
                (handler)(
                    ctx.clone(),
                    request.clone(),
                    Box::new({
                        let value = op.clone();
                        move |req| value(req).into_response()
                    }),
                )
            })));
        }

        // last, kick off the chain by calling the end of it (the outermost middleware)
        chain.last().unwrap()(request)
    }
}
