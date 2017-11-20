pub fn strip_leading_slash(route: String) -> String {
  match route.chars().nth(0) {
    Some(val) => {
      if val == '/' {
        (route[1..]).to_owned()
      } else {
        route
      }
    },
    None => route
  }
}
