use std::fs::File;
use std::io::prelude::*;
use std::env;
use lazy_static::*;
use chashmap::CHashMap;
use bytes::BytesMut;
use bytes::Bytes;
use std::iter::FromIterator;
use thruster_core::context::Context;
use thruster_proc::middleware_fn;
use thruster_core::{MiddlewareNext, MiddlewareReturnValue, MiddlewareResult, map_try};

lazy_static! {
  static ref CACHE: CHashMap<String, Vec<u8>> = {
    CHashMap::new()
  };
}

///
/// Middleware to set send a static file. Requires
/// `thruster_error_handling` to be turned on.
///
/// The `file` middleware takes the currently requested route
/// and checks the local filesystem for that file. If found,
/// then it returns the file, if not, then it returns a NotFound
/// error.
///
/// `file`, and the underlying `get_file`, also attempt to cache
/// any files that it reads. The underlying data is stored in a
/// CHashMap. This feature can be turned off by setting the env
/// var RUST_CACHE=off
///
#[middleware_fn]
pub async fn file<T: 'static + Context + Send>(
  mut context: T,
  _next: MiddlewareNext<T>,
) -> MiddlewareResult<T> {
    let content = map_try!(get_file(context.route()), Err(_) => Error::not_found_error(context));

    context.set_body_bytes(content);
    Ok(context)
}

///
/// An access point to the underlying cached file implementation
/// in case the developer wants to write custom parsing for the
/// path.
///
/// `get_file` attempts to cache any files that it reads. The
/// underlying data is stored in a CHashMap. This feature can
/// be turned off by setting the env var RUST_CACHE=off
///
pub fn get_file(path: &str) -> Result<Bytes, std::io::Error> {
  let path = path
    .replace("/static", "")
    .replace("..", "");

  let is_cache_off = env::var("RUST_CACHE")
    .unwrap_or_else(|_| "off".to_string()) == "off";

  let fetched = CACHE.get(&path);
  match (is_cache_off, fetched) {
    (false, Some(val)) => {
      Ok(Bytes::from(BytesMut::from_iter(&*val)))
    },
    (false, None) => {
      let val = read_file(&path)?;
      CACHE.insert(path.clone(), val);
      let val = CACHE.get(&path).unwrap();
      Ok(Bytes::from(BytesMut::from_iter(&*val)))
    },
    (true, _) => {
      let val = read_file(&path)?;
      Ok(Bytes::from(BytesMut::from_iter(&*val)))
    }

  }
}

fn read_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
  let mut file = File::open(format!("public{}", path))?;
  let mut contents = Vec::new();
  file.read_to_end(&mut contents)?;

  Ok(contents)
}
