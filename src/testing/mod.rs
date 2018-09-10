use app::App;
use context::Context;
use bytes::{BytesMut, BufMut};
use request::decode;
use futures::Future;
use std::collections::HashMap;
use response::{Response, StatusMessage};

pub fn get<T: Context + Send>(app: App<T>, route: &str) -> TestResponse {
  let body = format!("GET {} HTTP/1.1\nHost: localhost:8080\n\n", route);

  let mut bytes = BytesMut::with_capacity(body.len());
  bytes.put(&body);


  let request = decode(&mut bytes).unwrap().unwrap();
  let response = app.resolve(request).wait().unwrap();

  TestResponse::new(response)
}

pub fn delete<T: Context + Send>(app: App<T>, route: &str) -> TestResponse {
  let body = format!("DELETE {} HTTP/1.1\nHost: localhost:8080\n\n", route);

  let mut bytes = BytesMut::with_capacity(body.len());
  bytes.put(&body);


  let request = decode(&mut bytes).unwrap().unwrap();
  let response = app.resolve(request).wait().unwrap();

  TestResponse::new(response)
}

pub fn post<T: Context + Send>(app: App<T>, route: &str, content: &str) -> TestResponse {
  let body = format!("POST {} HTTP/1.1\nHost: localhost:8080\nContent-Length: {}\n\n{}", route, content.len(), content);

  let mut bytes = BytesMut::with_capacity(body.len());
  bytes.put(&body);


  let request = decode(&mut bytes).unwrap().unwrap();
  let response = app.resolve(request).wait().unwrap();

  TestResponse::new(response)
}

pub fn put<T: Context + Send>(app: App<T>, route: &str, content: &str) -> TestResponse {
  let body = format!("PUT {} HTTP/1.1\nHost: localhost:8080\nContent-Length: {}\n\n{}", route, content.len(), content);

  let mut bytes = BytesMut::with_capacity(body.len());
  bytes.put(&body);


  let request = decode(&mut bytes).unwrap().unwrap();
  let response = app.resolve(request).wait().unwrap();

  TestResponse::new(response)
}

pub fn update<T: Context + Send>(app: App<T>, route: &str, content: &str) -> TestResponse {
  let body = format!("UPDATE {} HTTP/1.1\nHost: localhost:8080\nContent-Length: {}\n\n{}", route, content.len(), content);

  let mut bytes = BytesMut::with_capacity(body.len());
  bytes.put(&body);


  let request = decode(&mut bytes).unwrap().unwrap();
  let response = app.resolve(request).wait().unwrap();

  TestResponse::new(response)
}

pub struct TestResponse {
  pub body: String,
  pub headers: HashMap<String, String>,
  pub status: (String, u32)
}

impl TestResponse {
  fn new(response: Response) -> TestResponse {
    let mut headers = HashMap::new();
    let header_string = String::from_utf8(response.header_raw.to_vec()).unwrap();

    for header_pair in header_string.split("\r\n") {
      if header_pair.len() > 0 {
        let mut split = header_pair.split(":");
        let key = split.next().unwrap().trim().to_owned();
        let value = split.next().unwrap().trim().to_owned();

        headers.insert(key, value);
      }
    }

    TestResponse {
      body: String::from_utf8(response.response).unwrap(),
      headers: headers,
      status: match response.status_message {
        StatusMessage::Ok => ("Ok".to_owned(), 200),
        StatusMessage::Custom(code, message) => (message, code)
      }
    }
  }
}
