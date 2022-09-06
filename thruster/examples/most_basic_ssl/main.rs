use log::info;
use thruster::ssl_server::SSLServer;
use thruster::ThrusterServer;
use thruster::{m, middleware_fn};
use thruster::{App, BasicContext as Ctx, Request};
use thruster::{MiddlewareNext, MiddlewareResult};

#[middleware_fn]
async fn plaintext(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let val = "Hello, World!";
    context.body(val);
    Ok(context)
}

#[middleware_fn]
async fn test_fn_404(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    context.body("404");
    Ok(context)
}

fn main() {
    env_logger::init();
    info!("Starting server...");

    let app = App::<Request, Ctx, ()>::new_basic()
        .get("/plaintext", m!(Ctx, [plaintext]))
        .set404(m!(Ctx, [test_fn_404]));

    let mut server = SSLServer::new(app);
    server.cert(include_bytes!("identity.p12").to_vec());
    server.cert_pass("asdfasdfasdf");
    server.start("0.0.0.0", 4321);
}
