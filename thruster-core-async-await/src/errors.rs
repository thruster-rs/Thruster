pub trait Error<C> {
  fn build_context(self: Box<Self>) -> C;
}
