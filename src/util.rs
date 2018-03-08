pub fn strip_leading_slash<'a>(route: &'a str) -> &'a str {
  match route.chars().nth(0) {
    Some(val) => {
      if val == '/' {
        &route[1..]
      } else {
        route
      }
    },
    None => route
  }
}

pub fn strip_trailing_slash<'a>(route: &'a str) -> &'a str {
  if route.len() < 2 {
    route
  } else {
    match route.chars().nth(route.len() - 1) {
      Some(val) => {
        if val == '/' {
          &route[0..(route.len()-1)]
        } else {
          route
        }
      },
      None => route
    }
  }
}
