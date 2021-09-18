use actix_web::dev::{AppService, Body, HttpServiceFactory};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use futures::TryFutureExt;
use http::StatusCode;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio_util::sync::ReusableBoxFuture;

use crate::app::App;
use crate::context::basic_actix_context::ActixRequest;
use crate::core::context::Context;
use crate::server::ThrusterServer;
use crate::Response;

struct _Error {
    _message: String,
}

pub struct ActixServer<T: 'static + Context + Clone + Send + Sync, S: 'static + Send> {
    app: Arc<App<ActixRequest, T, S>>,
}

impl<T, S> ThrusterServer for ActixServer<T, S>
where
    T: Context<Response = Response> + 'static + Clone + Send + Sync,
    S: 'static + Send + Sync,
{
    type Context = T;
    type Response = Response;
    type Request = ActixRequest;
    type State = S;

    fn new(mut app: App<ActixRequest, T, Self::State>) -> Self {
        app = app.commit();

        ActixServer { app: Arc::new(app) }
    }

    fn build(self, _host: &str, _port: u16) -> ReusableBoxFuture<()> {
        panic!("Can't use build for the actix runtime.")
    }

    fn start(self, host: &str, port: u16)
    where
        Self: Sized,
    {
        actix_rt::System::new().block_on(async move {
            let app = self.app.clone();

            let _ = actix_web::HttpServer::new(move || {
                actix_web::App::new()
                    .data(app.clone())
                    .service(ThrusterActixServiceFactory::<T, S>::default())
            })
            .bind(format!("{}:{}", host, port))
            .expect(&format!("Could not bind to address {}:{}", host, port))
            .run()
            .await;
        });
    }
}

struct ThrusterActixServiceFactory<T, S> {
    context: PhantomData<T>,
    state: PhantomData<S>,
}

impl<T, S> Default for ThrusterActixServiceFactory<T, S> {
    fn default() -> Self {
        ThrusterActixServiceFactory {
            context: PhantomData::default(),
            state: PhantomData::default(),
        }
    }
}

impl<T, S> HttpServiceFactory for ThrusterActixServiceFactory<T, S>
where
    T: Context<Response = Response> + 'static + Clone + Send + Sync,
    S: 'static + Send + Sync,
{
    fn register(self, config: &mut AppService) {
        async fn handler<T, S>(
            req: HttpRequest,
            payload: web::Payload,
            app: web::Data<Arc<App<ActixRequest, T, S>>>,
        ) -> Result<HttpResponse, Error>
        where
            T: Context<Response = Response> + 'static + Clone + Send + Sync,
            S: 'static + Send + Sync,
        {
            let actix_req = ActixRequest::new(req, payload).await;

            let response = app
                .clone()
                .match_and_resolve(actix_req)
                .map_err(|e| Error::from(e))
                .await?;

            let r = response;

            let response = HttpResponse::new(StatusCode::OK).set_body(Body::from(r.response));

            Ok::<_, Error>(response)
        }

        let resource = actix_web::Resource::new("*")
            .name("ThrusterActix")
            .route(web::get().to(handler::<T, S>))
            .route(web::post().to(handler::<T, S>))
            .route(web::put().to(handler::<T, S>))
            .route(web::head().to(handler::<T, S>))
            .route(web::patch().to(handler::<T, S>))
            .route(web::delete().to(handler::<T, S>))
            .route(web::trace().to(handler::<T, S>));

        actix_web::dev::HttpServiceFactory::register(resource, config)
    }
}
