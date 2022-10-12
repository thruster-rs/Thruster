use log::info;
use thruster::{m, middleware_fn};
use thruster::{App, BasicContext as Ctx, Request, Server, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = "Hello, World!";
    context.body(val);
    Ok(context)
}

#[middleware_fn]
async fn noop(context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    Ok(context)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting server...");

    let app = App::<Request, Ctx, ()>::new_basic().get("/plaintext", m![plaintext]);

    let server = Server::new(app);
    tokio::spawn(server.build("0.0.0.0", 4321));

    let healthcheck = App::<Request, Ctx, ()>::new_basic().get("/healthz", m![noop]);

    let healthcheck_server = Server::new(healthcheck);
    tokio::spawn(healthcheck_server.build("0.0.0.0", 8080));

    futures::future::pending().await
}
