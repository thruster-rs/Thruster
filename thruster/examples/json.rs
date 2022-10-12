use log::info;
use serde::{Deserialize, Serialize};
use thruster::{
    context::{
        basic_hyper_context::{generate_context, BasicHyperContext as Ctx, HyperRequest},
        context_ext::ContextExt,
    },
    errors::ThrusterError,
    hyper_server::HyperServer,
    m, middleware_fn, App, MiddlewareNext, MiddlewareResult, ThrusterServer,
};

#[derive(Deserialize)]
struct EchoJsonRequest {
    name: String,
}

#[derive(Serialize)]
struct EchoJsonResponse {
    message: String,
}

#[middleware_fn]
async fn echo_json(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let body: EchoJsonRequest = context.get_json::<EchoJsonRequest>().await.map_err(|e| {
        let e: ThrusterError<Ctx> = e.into();
        e
    })?;
    context.json(&EchoJsonResponse {
        message: format!("Hello, {}", body.name),
    })?;

    Ok(context)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting server...");

    HyperServer::new(
        App::<HyperRequest, Ctx, ()>::create(generate_context, ()).post("/json", m![echo_json]),
    )
    .build("0.0.0.0", 4321)
    .await;
}
