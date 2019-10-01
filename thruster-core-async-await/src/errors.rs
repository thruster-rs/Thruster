use std::error::Error as StdError;

#[derive(Debug)]
pub struct ThrusterError<C> {
  pub context: C,
  pub message: String,
  pub status: u32,
  pub cause: Option<Box<dyn StdError>>
}
