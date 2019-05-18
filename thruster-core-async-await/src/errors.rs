#[derive(Debug)]
pub struct ThrusterError<C> {
  pub context: C,
  pub message: String,
  pub status: u32
}
