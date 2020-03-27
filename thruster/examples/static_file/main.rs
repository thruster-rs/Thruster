///
/// Hey! This program is better with a few extra env vars. Try
/// RUST_HOST_DIR=./examples/static_file/ RUST_ROOT_DIR=/ \
///   cargo run --example static_file --features="hyper_server file"
///
/// RUST_HOST_DIR is where to serve the content from on your local
/// filesystem
///
/// RUST_ROOT_DIR is the root of where content should be requested
/// from. In otherwords, RUST_ROOT_DIR will attempt to replace the
/// first piece of a path with the RUST_HOST_DIR.
///
use hyper::Body;
use log::info;
use thruster::context::basic_hyper_context::{
    generate_context, to_owned_request, BasicHyperContext as Ctx, HyperRequest,
};
use thruster::hyper_server::HyperServer;
use thruster::middleware::file::{file, get_file};
use thruster::{async_middleware, middleware_fn};
use thruster::{App, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

#[middleware_fn]
async fn index(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let content = get_file("examples/static_file/index.html").unwrap();
    context.body = Body::from(content);
    Ok(context)
}

fn main() {
    env_logger::init();
    info!("Starting server...");

    let mut app = App::<HyperRequest, Ctx>::create(generate_context);
    app.get("/", async_middleware!(Ctx, [index]));
    app.get("/*", async_middleware!(Ctx, [to_owned_request, file]));
    let server = HyperServer::new(app);
    server.start("0.0.0.0", 4321);
}
