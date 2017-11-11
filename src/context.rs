use response::Response;

pub struct Context {
  pub body: String,
  pub response: Response,
  pub _identifier: u32
}

impl Context {
  pub fn new(response: Response) -> Context {
    Context {
      body: "".to_owned(),
      response: response,
      _identifier: 0
    }
  }
}
