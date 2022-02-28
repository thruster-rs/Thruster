use hyper::Body;
use log::info;
use thruster::context::hyper_request::HyperRequest;
use thruster::context::typed_hyper_context::TypedHyperContext;
use thruster::hyper_server::HyperServer;
use thruster::{async_middleware, middleware_fn};
use thruster::{App, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

type Ctx = TypedHyperContext<RequestConfig>;

struct ServerConfig {
    server_id: String,
}

struct RequestConfig {
    server_id: String,
    path: String,
}

fn generate_context(request: HyperRequest, state: &ServerConfig, path: &str) -> Ctx {
    Ctx::new(
        request,
        RequestConfig {
            server_id: state.server_id.clone(),
            path: path.to_string(),
        },
    )
}

#[middleware_fn]
async fn state_printer(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body = Body::from(format!(
        "server id: {}, requested path: {}",
        context.extra.server_id, context.extra.path
    ));
    Ok(context)
}

fn main() {
    env_logger::init();
    info!("Starting server...");

    let app = App::<HyperRequest, Ctx, ServerConfig>::create(
        generate_context,
        ServerConfig {
            server_id: "some-test-id".to_string(),
        },
    )
    .get("/a/:key1", async_middleware!(Ctx, [state_printer]))
    .get("/b/:key2", async_middleware!(Ctx, [state_printer]));

    let server = HyperServer::new(app);
    server.start("0.0.0.0", 4321);
}
