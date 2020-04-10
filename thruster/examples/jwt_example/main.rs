extern crate serde_json;
extern crate frank_jwt;

mod jwt_context;

use jwt_context::JwtContext;
use log::info;
use serde_json::{json, Value};
use thruster::middleware::cookies::HasCookies;
use thruster::{async_middleware, middleware_fn};
use thruster::{App, Request, Server, ThrusterServer};
use thruster::{MiddlewareNext, MiddlewareResult};

#[middleware_fn]
async fn jwt_handler(
    mut context: JwtContext,
    next: MiddlewareNext<JwtContext>,
) -> MiddlewareResult<JwtContext> {
    let jwt = context.headers().get("Authorization");

    if jwt.is_none() {
        context.body("{{\"message\": \"Authorization header required\",\"success\":false}}");
        context.status(401); // Unauthorized

        return Ok(context);
    }
    let jwt = jwt.unwrap();

    let (_, payload) = frank_jwt::decode(
        &jwt,
        &"secret123".to_string(),
        frank_jwt::Algorithm::HS256,
        &frank_jwt::ValidationOptions::default(),
    )
    .unwrap();

    context.user_id = payload
        .get("userId")
        .unwrap_or(&Value::String("undefined".to_string()))
        .to_string();

    next(context).await
}

#[middleware_fn]
async fn user_data(
    mut context: JwtContext,
    _next: MiddlewareNext<JwtContext>,
) -> MiddlewareResult<JwtContext> {
    context.body(&format!(
        "Hello there. Looks like you are user {}",
        context.user_id
    ));

    Ok(context)
}

#[middleware_fn]
async fn user_login(
    mut context: JwtContext,
    _next: MiddlewareNext<JwtContext>,
) -> MiddlewareResult<JwtContext> {
    let body: Value = serde_json::from_str(&context.get_body()).unwrap();

    // I hope to god this isn't your password. If it is, please reconsider your password policy
    if body.get("username") == Some(&Value::String("test_user".to_string()))
        && body.get("password") == Some(&Value::String("pass123".to_string()))
    {
        // Authentication  done. Generate JWT now

        // JWT generation taken from:
        // https://github.com/GildedHonour/frank_jwt

        let mut payload = json!({
            "userId": "test_user",
            "allowed_resources": ["test_values","print_hello","sleep","cry","repeat"]
        });
        let mut header = json!({});
        let secret = "secret123";
        let jwt = frank_jwt::encode(
            header,
            &secret.to_string(),
            &payload,
            frank_jwt::Algorithm::HS256,
        )
        .unwrap();

        context.body(&format!("{{\"success\": true, \"jwt\": \"{}\"}}", jwt));

        Ok(context)
    } else {
        context.body(
            "{{\"success\": false, \"message\": \"Wrong credentials. Use test_user and pass123\"}}",
        );

        Ok(context)
    }
}

#[middleware_fn]
async fn four_oh_four(
    mut context: JwtContext,
    _next: MiddlewareNext<JwtContext>,
) -> MiddlewareResult<JwtContext> {
    context.status(404);
    context.body("Whoops! That route doesn't exist!");
    Ok(context)
}

fn main() {
    env_logger::init();
    info!("Starting server...");

    let mut app = App::<Request, JwtContext>::new_basic();

    app.use_middleware("/data", async_middleware!(JwtContext, [jwt_handler]));

    app.get("/data", async_middleware!(JwtContext, [user_data]));
    app.get("/login", async_middleware!(JwtContext, [user_login]));
    app.set404(async_middleware!(JwtContext, [four_oh_four]));

    let server = Server::new(app);
    server.start("0.0.0.0", 4321);
}
