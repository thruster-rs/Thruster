pub struct ProcessedRoute {
  pub head: String,
  pub tail: Option<String>
}

pub fn process_route(route: &str) -> Option<ProcessedRoute> {
  if route.len() == 0 {
    return None
  }

  let mut split = route.split("/");
  match split.next() {
    Some(head) => {
      match split.next() {
        Some(tail) => {
          let mut joined = tail.to_owned();

          for part in split {
            joined = format!("{}/{}", joined, part);
          }

          if head == "" {
            return process_route(&joined)
          }

          Some(ProcessedRoute {
            head: head.to_owned(),
            tail: Some(joined)
          })
        },
        None => Some(ProcessedRoute {
          head: head.to_owned(),
          tail: None
        })
      }
    },
    None => None
  }
}
