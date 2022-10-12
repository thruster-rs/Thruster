use hyper::Body;
use log::info;
use thruster::context::basic_hyper_context::{
    generate_context, BasicHyperContext as Ctx, HyperRequest,
};
use thruster::hyper_server::HyperServer;
use thruster::middleware::cors::cors;
use thruster::{m, middleware_fn};
use thruster::{App, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = "Hello, World!";
    context.body = Body::from(val);
    Ok(context)
}

fn main() {
    env_logger::init();
    info!("Starting server...");

    let mut app = App::<HyperRequest, Ctx, ()>::create(generate_context, ())
        .middleware("/", m![cors])
        .get("/plaintext", m![plaintext]);

    app.connection_timeout = 5000;

    let server = HyperServer::new(app);
    server.start("0.0.0.0", 4321);
}
