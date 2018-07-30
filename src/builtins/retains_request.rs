use request::Request;

pub trait RetainsRequest {
  fn get_request<'a>(&'a self) -> &'a Request;
}
