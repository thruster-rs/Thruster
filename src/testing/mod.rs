use app::App;
use context::Context;
use bytes::{BytesMut, BufMut};
use request::{decode};
use futures::{future, Future};

pub fn get<T: Context + Send>(app: App<T>, route: &str) -> String {
  let body = format!("GET {} HTTP/1.1\nHost: localhost:8080\n\n", route);

  let mut bytes = BytesMut::with_capacity(body.len());
  bytes.put(&body);


  let request = decode(&mut bytes).unwrap().unwrap();
  let response = app.resolve(request).wait().unwrap();

  String::from_utf8(response.response).unwrap()
}

pub fn delete<T: Context + Send>(app: App<T>, route: &str) -> String {
  let body = format!("DELETE {} HTTP/1.1\nHost: localhost:8080\n\n", route);

  let mut bytes = BytesMut::with_capacity(body.len());
  bytes.put(&body);


  let request = decode(&mut bytes).unwrap().unwrap();
  let response = app.resolve(request).wait().unwrap();

  String::from_utf8(response.response).unwrap()
}

pub fn post<T: Context + Send>(app: App<T>, route: &str, content: &str) -> String {
  let body = format!("POST {} HTTP/1.1\nHost: localhost:8080\nContent-Length: {}\n\n{}", route, content.len(), content);

  let mut bytes = BytesMut::with_capacity(body.len());
  bytes.put(&body);


  let request = decode(&mut bytes).unwrap().unwrap();
  let response = app.resolve(request).wait().unwrap();

  String::from_utf8(response.response).unwrap()
}

pub fn put<T: Context + Send>(app: App<T>, route: &str, content: &str) -> String {
  let body = format!("PUT {} HTTP/1.1\nHost: localhost:8080\nContent-Length: {}\n\n{}", route, content.len(), content);

  let mut bytes = BytesMut::with_capacity(body.len());
  bytes.put(&body);


  let request = decode(&mut bytes).unwrap().unwrap();
  let response = app.resolve(request).wait().unwrap();

  String::from_utf8(response.response).unwrap()
}

pub fn update<T: Context + Send>(app: App<T>, route: &str, content: &str) -> String {
  let body = format!("UPDATE {} HTTP/1.1\nHost: localhost:8080\nContent-Length: {}\n\n{}", route, content.len(), content);

  let mut bytes = BytesMut::with_capacity(body.len());
  bytes.put(&body);


  let request = decode(&mut bytes).unwrap().unwrap();
  let response = app.resolve(request).wait().unwrap();

  String::from_utf8(response.response).unwrap()
}
