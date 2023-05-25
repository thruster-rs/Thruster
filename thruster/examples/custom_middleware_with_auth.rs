use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use log::info;
use thruster::{
    context::typed_hyper_context::TypedHyperContext,
    errors::{ErrorSet, ThrusterError},
    hyper_server::HyperServer,
    m,
    middleware::cookies::HasCookies,
    middleware_fn, App, HyperRequest, MiddlewareNext, MiddlewareResult, ThrusterServer,
};
use thruster_jab::{fetch, provide, JabDI};
use thruster_proc::context_state;
use tokio::sync::Mutex;

type Ctx = TypedHyperContext<State>;

context_state!(State => [Arc<JabDI>, Option<User>]);

#[derive(Default)]
struct ServerConfig {
    di: Arc<JabDI>,
}

fn generate_context(request: HyperRequest, state: &ServerConfig, _path: &str) -> Ctx {
    Ctx::new(request, (state.di.clone(), None))
}

#[middleware_fn]
async fn hello(mut context: Ctx, _next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let user: &Option<User> = context.extra.get();
    let user = user.as_ref().unwrap();

    context.body(&format!("Hello, {} {}", user.first, user.last));

    Ok(context)
}

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Starting server...");

    let mut jab = JabDI::default();
    provide!(jab, dyn AuthProvider, FakeAuthenticator::default());

    let app = App::<HyperRequest, Ctx, ServerConfig>::create(
        generate_context,
        ServerConfig { di: Arc::new(jab) },
    )
    .get("/hello", m![authenticate, hello]);

    let server = HyperServer::new(app);
    server.build("0.0.0.0", 4321).await;
}

// This stection starts the custom middleawre
pub struct User {
    first: String,
    last: String,
}

#[async_trait]
trait AuthProvider {
    async fn authenticate(&self, session_token: &str) -> Result<User, ()>;
}

struct FakeAuthenticator {
    fake_db: Mutex<HashMap<String, User>>,
}

impl Default for FakeAuthenticator {
    fn default() -> Self {
        Self {
            fake_db: Mutex::new(HashMap::from([
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
            ])),
        }
    }
}

#[async_trait]
impl AuthProvider for FakeAuthenticator {
    async fn authenticate(&self, session_token: &str) -> Result<User, ()> {
        self.fake_db.lock().await.remove(session_token).ok_or(())
    }
}

#[middleware_fn]
async fn authenticate(mut context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx> {
    let di: &Arc<JabDI> = context.extra.get();
    let auth = fetch!(di, dyn AuthProvider);

    let session_header = context
        .get_header("Session-Token")
        .pop()
        .unwrap_or_default();

    let user = auth
        .authenticate(&session_header)
        .await
        .map_err(|_| ThrusterError::unauthorized_error(Ctx::default()))?;

    *context.extra.get_mut() = Some(user);

    next(context).await
}
