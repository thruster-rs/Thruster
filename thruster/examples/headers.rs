use hyper::Body;
use log::info;
use thruster::{
    context::{
        basic_hyper_context::{generate_context, BasicHyperContext as Ctx, HyperRequest},
        context_ext::ContextExt,
    },
    hyper_server::HyperServer,
    m, middleware_fn, App, Context, MiddlewareNext, MiddlewareResult, ThrusterServer,
};

#[middleware_fn]
async fn echo_header(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.set("Content-Type", "text/plain");
    context.body = Body::from(format!(
        "Hello, {}",
        context
            .get_req_header("Custom-Header")
            .unwrap_or("No 'Custom-Header' present")
    ));

    Ok(context)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting server...");

    HyperServer::new(
        App::<HyperRequest, Ctx, ()>::create(generate_context, ())
            .post("/echo/:name", m![echo_header]),
    )
    .build("0.0.0.0", 4321)
    .await;
}
