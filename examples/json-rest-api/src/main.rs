mod handlers;
mod models;
mod schema;

use snx::router;

struct App;

impl snx::App for App {
    fn with_routes(builder: router::Builder) -> router::Router {
        builder
            .get("/", handlers::posts::index)
            .prefix("/posts", |router| {
                router
                    .post("/", handlers::posts::store)
                    .get("/", handlers::posts::index)
                    .get("/{id}", handlers::posts::get)
                    .put("/{id}", handlers::posts::update)
                    .delete("/{id}", handlers::posts::destroy)
            })
            .build()
            .unwrap()
    }
}

fn main() {
    snx::boot::<App>();
}
