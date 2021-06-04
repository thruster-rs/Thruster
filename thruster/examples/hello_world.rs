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

fn main() {
    env_logger::init();
    info!("Starting server...");

    let mut app = App::<Request, Ctx, ()>::new_basic();

    app.get("/plaintext", m![plaintext]);

    let server = Server::new(app);
    server.start("0.0.0.0", 4321);
}
