use std::collections::HashMap;

use crate::app::App;

use crate::core::context::Context;
use crate::context::hyper_request::HyperRequest;
use hyper::{body, Body, Request, Response};

pub async fn request<T: Context<Response = Response<Body>> + Send, S: Send>(
    app: &App<HyperRequest, T, S>,
    request: Request<Body>,
) -> TestResponse {
    let matched_route = app.resolve_from_method_and_path(request.method().as_str(), &request.uri().to_string());
    let response = app.resolve(HyperRequest::new(request), matched_route).await.unwrap();

    TestResponse::new(response).await
}

#[derive(Debug)]
pub struct TestResponse {
    pub body: String,
    pub headers: HashMap<String, String>,
    pub status: u16,
}

impl TestResponse {
    async fn new(response: Response<Body>) -> TestResponse {
        let mut headers = HashMap::new();

        for (key, value) in response.headers().iter() {
          headers.insert(key.as_str().to_owned(), value.to_str().unwrap().to_owned());
        }

        let status = response.status().as_u16();
        TestResponse {
            body: std::str::from_utf8(&body::to_bytes(response.into_body()).await.unwrap().to_vec()).unwrap().to_string(),
            headers,
            status,
        }
    }
}
