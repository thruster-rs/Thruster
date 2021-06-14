use hyper::Body;
use log::info;
use thruster::context::basic_hyper_context::{
    generate_context, BasicHyperContext as Ctx, HyperRequest,
};
use thruster::hyper_server::HyperServer;
use thruster::{m, middleware_fn};
use thruster::{App, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = "Hello, World!";
    context.body = Body::from(val);
    Ok(context)
}

#[middleware_fn]
async fn nested_plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = "(nested) Hello, World!";
    context.body = Body::from(val);
    Ok(context)
}

fn main() {
    env_logger::init();
    info!("Starting server...");

    let mut app = App::<HyperRequest, Ctx, ()>::create(generate_context, ());
    app.get("/plaintext", m![plaintext]);

    let mut nested_app = App::<HyperRequest, Ctx, ()>::create(generate_context, ());
    nested_app.get("/plaintext", m![nested_plaintext]);

    app.use_sub_app("/nested", nested_app);

    let server = HyperServer::new(app);
    server.start("0.0.0.0", 4321);
}
