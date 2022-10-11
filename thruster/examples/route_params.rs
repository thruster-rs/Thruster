use hyper::Body;
use log::info;
use thruster::{
    context::{
        basic_hyper_context::{generate_context, BasicHyperContext as Ctx, HyperRequest},
        context_ext::ContextExt,
    },
    errors::{ErrorSet, ThrusterError},
    hyper_server::HyperServer,
    m, middleware_fn, App, Context, MiddlewareNext, MiddlewareResult, ThrusterServer,
};

#[middleware_fn]
async fn echo_route_params(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.set("Content-Type", "text/plain");
    context.body = Body::from(format!(
        "Hello, {}",
        context
            .get_params()
            .get("name")
            .ok_or(ThrusterError::generic_error(Ctx::default()))?
            .param
    ));

    Ok(context)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting server...");

    HyperServer::new(
        App::<HyperRequest, Ctx, ()>::create(generate_context, ())
            .post("/echo/:name", m![echo_route_params]),
    )
    .build("0.0.0.0", 4321)
    .await;
}
