use snx::router;

struct App;

impl snx::App for App {
    fn with_routes(builder: router::Builder) -> router::Router {
        builder
            .get("/", |_| "hello world!")
            .build()
            .unwrap()
    }
}

fn main() {
    snx::boot::<App>();
}
