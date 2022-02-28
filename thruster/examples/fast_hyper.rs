use hyper::Body;
use log::info;
use thruster::context::fast_hyper_context::{
    generate_context, FastHyperContext as Ctx, HyperRequest,
};
use thruster::hyper_server::HyperServer;
use thruster::{async_middleware, middleware_fn};
use thruster::{App, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = "Hello, World!";
    context.body = Some(Body::from(val));
    Ok(context)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting server...");

    let app = App::<HyperRequest, Ctx, ()>::create(generate_context, ())
        .get("/plaintext", async_middleware!(Ctx, [plaintext]));

    let server = HyperServer::new(app);
    server.build("0.0.0.0", 4321).await;
}
