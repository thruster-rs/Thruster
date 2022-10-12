use hyper::Body;
use log::info;
use thruster::{
    context::basic_hyper_context::{generate_context, BasicHyperContext as Ctx, HyperRequest},
    hyper_server::HyperServer,
    m,
    middleware::query_params::query_params,
    middleware_fn, App, Context, MiddlewareNext, MiddlewareResult, ThrusterServer,
};

#[middleware_fn]
async fn echo_query(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.set("Content-Type", "text/plain");
    context.body = Body::from(format!(
        "Hello, {}",
        context
            .query_params
            .get("name")
            .map(Clone::clone)
            .unwrap_or_else(|| "Unknown".to_string())
    ));

    Ok(context)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting server...");

    HyperServer::new(
        App::<HyperRequest, Ctx, ()>::create(generate_context, ())
            .post("/query", m![query_params, echo_query]),
    )
    .build("0.0.0.0", 4321)
    .await;
}
