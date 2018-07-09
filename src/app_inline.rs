#[macro_export]
macro_rules! app_inline {
  ($app:expr) => (
    let root_node = $app._route_parser.route_tree.root_node;

    println!("app: {}", root_node.to_string(""));
    for (route, middleware) in root_node.enumerate() {
      println!("{}: {}", route, middleware.len());
    }
  )
  // ( $head_template:expr $(;$key:expr; $template:expr)* ) => {
  //   {
  //     let mut total_length = 0;
  //     total_length = total_length + $head_template.len();

  //     $(
  //       total_length = total_length + $key.len() + $template.len();
  //     )*

  //     let mut output_string = String::with_capacity(total_length);
  //     output_string.push_str($head_template);

  //     $(
  //       output_string.push_str($key);
  //       output_string.push_str($template);
  //     )*

  //     output_string
  //   }
  // }
}
