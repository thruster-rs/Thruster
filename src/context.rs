use response::Response;
use request::Request;

pub struct Context {
  pub response: Response,
  pub _identifier: u32
}

impl Context {
  pub fn new(response: Response) -> Context {
    Context {
      response: response,
      _identifier: 0
    }
  }
}
