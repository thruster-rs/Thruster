use regex::Regex;

lazy_static! {
  static ref WILDCARD_PARAM_REGEX: Regex = Regex::new(r":\w+").unwrap();
}

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

pub fn wildcardify_params(route: &str) -> String {
  let result = WILDCARD_PARAM_REGEX.replace_all(route, "*");

  result.into_owned()
}
