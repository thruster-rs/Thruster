use std::sync::Arc;

use authentication_module::{authenticate, User, AuthProvider, FakeAuthenticator};
use log::info;
use thruster::{
    context::typed_hyper_context::TypedHyperContext,
    hyper_server::HyperServer,
    m,
    middleware_fn, App, HyperRequest, MiddlewareNext, MiddlewareResult, ThrusterServer,
};
use thruster_jab::{provide, JabDI};
use thruster_proc::context_state;

type Ctx = TypedHyperContext<State>;

#[derive(Default)]
#[context_state]
pub struct State(Option<Arc<JabDI>>, Option<User>);

#[derive(Default)]
struct ServerConfig {
    di: Arc<JabDI>,
}

fn generate_context(request: HyperRequest, state: &ServerConfig, _path: &str) -> Ctx {
    Ctx::new(request, State(Some(state.di.clone()), None))
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

// This stection starts the custom middleware
mod authentication_module {
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::sync::Arc;
    
    use thruster::{
        context::typed_hyper_context::TypedHyperContext,
        errors::{ErrorSet, ThrusterError},
        middleware::cookies::HasCookies,
        middleware_fn, MiddlewareNext, MiddlewareResult, ContextState,
    };
    use thruster_jab::{fetch, JabDI};
    use tokio::sync::Mutex;

    #[derive(Clone)]
    pub struct User {
        pub first: String,
        pub last: String,
    }
    
    #[async_trait]
    pub trait AuthProvider {
        async fn authenticate(&self, session_token: &str) -> Result<User, ()>;
    }
    
    pub struct FakeAuthenticator {
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
    pub async fn authenticate<S: Send + Sync + Default + ContextState<Option<Arc<JabDI>>> + ContextState<Option<User>>>(mut context: TypedHyperContext<S>, next: MiddlewareNext<TypedHyperContext<S>>) -> MiddlewareResult<TypedHyperContext<S>> {
        let di: &Option<Arc<JabDI>> = context.extra.get();
        let auth = fetch!(di.as_ref().unwrap(), dyn AuthProvider);
    
        let session_header = context
            .get_header("Session-Token")
            .pop()
            .unwrap_or_default();
    
        let user = auth
            .authenticate(&session_header)
            .await
            .map_err(|_| ThrusterError::unauthorized_error(TypedHyperContext::default()))?;
    
        *context.extra.get_mut() = Some(user);
    
        next(context).await
    }    
}
