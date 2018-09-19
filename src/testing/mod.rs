use app::App;
use context::Context;
use futures::Future;
use std::collections::HashMap;

use hyper::{Body, Response, Request};
use http::Method;
use futures::Stream;

pub fn get<T: Context + Send>(app: App<T>, route: &str) -> TestResponse {
  let mut request_builder = Request::builder();
  request_builder.method(Method::GET);
  request_builder.uri(route);

  let response = app.resolve(request_builder.body(Body::empty()).unwrap()).wait().unwrap();

  TestResponse::new(response)
}

pub fn delete<T: Context + Send>(app: App<T>, route: &str) -> TestResponse {
  let mut request_builder = Request::builder();
  request_builder.method(Method::DELETE);
  request_builder.uri(route);

  let response = app.resolve(request_builder.body(Body::empty()).unwrap()).wait().unwrap();


  TestResponse::new(response)
}

pub fn post<T: Context + Send>(app: App<T>, route: &str, content: &str) -> TestResponse {
  let mut request_builder = Request::builder();
  request_builder.method(Method::POST);
  request_builder.uri(route);

  let response = app.resolve(request_builder.body(Body::from(content.to_owned())).unwrap()).wait().unwrap();

  TestResponse::new(response)
}

pub fn put<T: Context + Send>(app: App<T>, route: &str, content: &str) -> TestResponse {
  let mut request_builder = Request::builder();
  request_builder.method(Method::PUT);
  request_builder.uri(route);

  let response = app.resolve(request_builder.body(Body::from(content.to_owned())).unwrap()).wait().unwrap();


  TestResponse::new(response)
}

pub fn patch<T: Context + Send>(app: App<T>, route: &str, content: &str) -> TestResponse {
  let mut request_builder = Request::builder();
  request_builder.method(Method::PATCH);
  request_builder.uri(route);

  let response = app.resolve(request_builder.body(Body::from(content.to_owned())).unwrap()).wait().unwrap();


  TestResponse::new(response)
}

pub struct TestResponse {
  pub body: String,
  pub headers: HashMap<String, String>,
  pub status: String
}

impl TestResponse {
  fn new(response: Response<Body>) -> TestResponse {
    let mut headers = HashMap::new();

    for (name, value) in response.headers() {
      headers.insert(name.as_str().to_owned(),
        value.to_str().unwrap().to_owned());
    }

    let status = response.status();


    let body = response.into_body()
      .concat2()
      .map_err(|_err| ())
      .map(|chunk| {
        let v = chunk.to_vec();
        String::from_utf8(v).unwrap()
      })
      .wait()
      .unwrap();

    TestResponse {

      body: body,
      headers: headers,
      status: status.as_str().to_owned()
    }
  }
}
