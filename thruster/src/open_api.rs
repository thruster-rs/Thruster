use serde::{de::DeserializeOwned, Serialize};

use crate::{app::ReturnValue, parser::middleware_traits::MiddlewareTuple, App, Context, ThrusterRequest};

struct Route {
    path: String,
    method: String,
    request: String,
    response: String,
    error: String
}

struct OpenAPIApp<R: 'static + ThrusterRequest, T: 'static + Context + Clone + Send + Sync, S: 'static + Send> {
    app: App<R, T, S>,
    routes: Vec<Route>
}

impl<R: 'static + ThrusterRequest, T: 'static + Context + Clone + Send + Sync, S: 'static + Send> OpenAPIApp<R, T, S> {
    fn new(app: App<R, T, S>) -> OpenAPIApp<R, T, S> {
        OpenAPIApp {
            app,
            routes: vec![]
        }
    }

    pub fn middleware(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self
    where
        T: Clone,
    {
        self.app = self.app.middleware(path, middlewares);
        self
    }

    pub fn router(mut self, prefix: &str, app: App<R, T, S>) -> Self {
        self.app = self.app.router(prefix, app);
        self
    }

    pub fn get<Req: DeserializeOwned, Res: Serialize, Err>(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self {
        self.app = self.app.get(path, m![|ctx, next| async {
            let ctx = ctx;
            let ctx = ctx.set_body(serde_json::to_string(&ctx).unwrap());
            next(ctx).await
            
        }].combine(middlewares));
        self.routes.push(Route {
            path: path.to_string(),
            method: "GET".to_string(),
            request: std::any::type_name::<Req>().to_string(),
            response: std::any::type_name::<Res>().to_string(),
            error: std::any::type_name::<Err>().to_string()
        });
        self
    }

    pub fn options(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self {
        self.app = self.app.options(path, middlewares);
        self
    }

    pub fn post(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self {
        self.app = self.app.post(path, middlewares);
        self
    }

    pub fn put(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self {
        self.app = self.app.put(path, middlewares);
        self
    }

    pub fn delete(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self {
        self.app = self.app.delete(path, middlewares);
        self
    }

    pub fn patch(mut self, path: &str, middlewares: MiddlewareTuple<ReturnValue<T>>) -> Self {
        self.app = self.app.patch(path, middlewares);
        self
    }
}

#[cfg(test)]
mod tests {
    use thruster_proc::{m, middleware_fn};
    use crate::{self as thruster, Request};

    use crate::{BasicContext, MiddlewareNext, MiddlewareResult};

    use super::*;
    use super::super::parser::middleware_traits::{MiddlewareFnPointer, MiddlewareTuple, ToTuple};

    struct Req;
    struct Res;
    struct Err;

    #[test]
    fn test_get_method_pushes_route_to_routes_array() {
        #[middleware_fn]
        async fn test_fn(ctx: BasicContext, _next: MiddlewareNext<BasicContext>) -> MiddlewareResult<BasicContext> {
            Ok(ctx)
        }

        let app = App::<Request, BasicContext, ()>::new_basic();
        let open_api_app = OpenAPIApp::new(app);

        let path = "/users";    

        let updated_open_api_app = open_api_app.get::<Req, Res, Err>(path, m![test_fn]);

        assert_eq!(updated_open_api_app.routes.len(), 1);
        assert_eq!(updated_open_api_app.routes[0].path, path);
        assert_eq!(updated_open_api_app.routes[0].method, "GET");
        assert_eq!(updated_open_api_app.routes[0].request, "thruster::open_api::tests::Req".to_string());
        assert_eq!(updated_open_api_app.routes[0].response, "thruster::open_api::tests::Res".to_string());
        assert_eq!(updated_open_api_app.routes[0].error, "thruster::open_api::tests::Err".to_string());
    }
}