use hyper::{Body, Request};

pub trait RetainsRequest {
  fn get_request<'a>(&'a self) -> &'a Request<Body>;
}
