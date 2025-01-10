use snx::{
    router::{Route, Router},
    Response,
};

struct App;

impl snx::App for App {
    fn with_routes() -> Router {
        Router::builder()
            .add_route(Route::get("/", |_| Response::builder().body(()).unwrap()))
            .build()
    }
}

fn main() {
    snx::boot::<App>().unwrap();
}
