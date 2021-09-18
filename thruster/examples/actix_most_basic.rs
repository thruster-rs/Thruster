use log::info;
use thruster::actix_server::ActixServer;
use thruster::context::basic_actix_context::{
    generate_context, ActixRequest, BasicActixContext as Ctx,
};
use thruster::{m, middleware_fn};
use thruster::{App, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body("Hello, world!");
    Ok(context)
}

fn main() {
    env_logger::init();
    info!("Starting server...");

    let mut app = App::<ActixRequest, Ctx, ()>::create(generate_context, ());
    app.get("/plaintext", m![plaintext]);

    let server = ActixServer::new(app);
    server.start("0.0.0.0", 4321);
}
