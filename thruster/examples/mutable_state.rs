use hyper::Body;
use log::info;
use thruster::context::hyper_request::HyperRequest;
use thruster::context::typed_hyper_context::TypedHyperContext;
use thruster::hyper_server::HyperServer;
use thruster::{m, middleware_fn};
use thruster::{App, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

use std::sync::Arc;
use std::sync::RwLock;

type Ctx = TypedHyperContext<RequestConfig>;

struct ServerConfig {
    val: Arc<RwLock<String>>,
}

struct RequestConfig {
    latest_value: Arc<RwLock<String>>,
}

fn generate_context(request: HyperRequest, state: &ServerConfig, _path: &str) -> Ctx {
    Ctx::new(
        request,
        RequestConfig {
            latest_value: state.val.clone(),
        },
    )
}

#[middleware_fn]
async fn state_setter(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let latest_value = context.extra.latest_value.clone();
    let mut latest_value = latest_value.write().unwrap();
    context.body = Body::from(format!("last value: {}", latest_value));
    *latest_value = context
        .hyper_request
        .as_ref()
        .unwrap()
        .params
        .get("val")
        .unwrap()
        .param
        .clone();

    Ok(context)
}

#[middleware_fn]
async fn state_getter(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let latest_value = context.extra.latest_value.clone();
    let latest_value = latest_value.read().unwrap();
    context.body = Body::from(format!("current value: {}", latest_value));

    Ok(context)
}

fn main() {
    env_logger::init();
    info!("Starting server...");

    let app = App::<HyperRequest, Ctx, ServerConfig>::create(
        generate_context,
        ServerConfig {
            val: Arc::new(RwLock::new("original".to_string())),
        },
    )
    .get("/set-value/:val", m![state_setter])
    .get("/get-value", m![state_getter]);

    let server = HyperServer::new(app);
    server.start("0.0.0.0", 4321);
}
