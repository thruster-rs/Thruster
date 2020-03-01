use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use crate::core::context::Context;

pub fn file<T: Context>(mut context: T, file_name: &str) -> T {
    let file = File::open(file_name).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();

    let _ = buf_reader.read_to_end(&mut contents);

    context.set_body(contents);
    context
}
