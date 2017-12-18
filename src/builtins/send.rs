use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use context::Context;

pub fn file<T: Context>(mut context: T, file_name: &str) -> T {
  let file = File::open(file_name).unwrap();
  let mut buf_reader = BufReader::new(file);
  let mut contents = String::new();

  buf_reader.read_to_string(&mut contents).unwrap();

  context.set_body(contents);
  context
}
