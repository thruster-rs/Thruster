use std::collections::HashMap;
use std::sync::Arc;

use log::info;
use thruster::{
    context::typed_hyper_context::TypedHyperContext,
    errors::{ErrorSet, ThrusterError},
    hyper_server::HyperServer,
    m,
    middleware::cookies::HasCookies,
    middleware_fn, App, HyperRequest, MiddlewareNext, MiddlewareResult, ThrusterServer,
};
use tokio::sync::Mutex;

type Ctx = TypedHyperContext<RequestConfig>;

struct User {
    first: String,
    last: String,
}

#[derive(Default)]
struct ServerConfig {
    fake_db: Arc<Mutex<HashMap<String, User>>>,
}

#[derive(Default)]
struct RequestConfig {
    fake_db: Arc<Mutex<HashMap<String, User>>>,
    user: Option<User>,
}

fn generate_context(request: HyperRequest, state: &ServerConfig, _path: &str) -> Ctx {
    Ctx::new(
        request,
        RequestConfig {
            fake_db: state.fake_db.clone(),
            user: None,
        },
    )
}

#[middleware_fn]
async fn hello(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let user = context.extra.user.as_ref().unwrap();

    context.body(&format!("Hello, {} {}", user.first, user.last));

    Ok(context)
}

#[middleware_fn]
async fn authenticate(mut context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let db = context.extra.fake_db.clone();

    let session_header = context
        .get_header("Session-Token")
        .pop()
        .unwrap_or_default();

    let user = db.lock().await.remove(&session_header);

    match user {
        Some(user) => {
            context.extra.user = Some(user);
            next(context).await
        }
        None => Err(ThrusterError::unauthorized_error(context)),
    }
}

fn main() {
    env_logger::init();
    info!("Starting server...");

    let app = App::<HyperRequest, Ctx, ServerConfig>::create(
        generate_context,
        ServerConfig {
            fake_db: Arc::new(Mutex::new(HashMap::from([
                (
                    "lukes-secret-session-token".to_string(),
                    User {
                        first: "Luke".to_string(),
                        last: "Skywalker".to_string(),
                    },
                ),
                (
                    "vaders-secret-session-token".to_string(),
                    User {
                        first: "Anakin".to_string(),
                        last: "Skywalker".to_string(),
                    },
                ),
            ]))),
        },
    )
    .get("/hello", m![authenticate, hello]);

    let server = HyperServer::new(app);
    server.start("0.0.0.0", 4321);
}
